#![feature(isqrt)]

mod settings;
mod data_trait;

use clap::Parser;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::os::unix::prelude::ExitStatusExt;
use std::path::Path;
use std::process::Output;
use std::process::Stdio;
use std::{fs, process};
use std::cmp::min;
use crate::data_trait::{DataTraits, MinimizationBytes, Mode};
use crate::settings::Settings;


fn main() {
    let mut settings = Settings::parse();
    settings.command = settings.command.replace("\"", "'");

    println!("Example command: {}", create_command(&settings, "test.jpg"));

    let initial_file_content = load_content(&settings);

    println!("Initial file size: {}", initial_file_content.len());

    let mut mb = MinimizationBytes {
        bytes: initial_file_content.clone(),
    };

    let (is_initially_broken, initial_output) = check_if_is_broken(&mb, &settings);
    if !is_initially_broken {
        eprintln!("File is not broken, check command or file");
        eprintln!("==================OUTPUT==================");
        eprintln!("{initial_output}");
        eprintln!("===========================================");
        process::exit(1);
    }

    let mut rng = rand::thread_rng();
    let mut iters = 0;

    while iters < settings.attempts {
        extend_results(
            remove_content_from_middle(&mut mb, &mut rng, &settings),
            &mut iters,
            Mode::Bytes,
        );
        extend_results(
            remove_some_content_from_start_end(&mut mb, &mut rng, &settings, true),
            &mut iters,
            Mode::Bytes,
        );
        extend_results(
            remove_some_content_from_start_end(&mut mb, &mut rng, &settings, false),
            &mut iters,
            Mode::Bytes,
        );
    }

    match mb.save_to_file(&settings.output_file) {
        Ok(_) => {
            println!("File was minimized to {}, after {} iterations", &settings.output_file, iters);
        }
        Err(e) => {
            eprintln!("Error writing file {}, reason {}", &settings.output_file, e);
            process::exit(1);
        }
    }
}

fn extend_results(result: (bool, u32, usize, usize), iterations_counter: &mut u32, mode: Mode) {
    let (changed, iterations, old_len, new_len) = result;
    *iterations_counter += iterations;
    if changed {
        println!("File was changed from {} to {} {}", old_len, new_len, mode);
    }
}

fn remove_content_from_middle<T>(
    content: &mut dyn DataTraits<T>,
    thread_rng: &mut ThreadRng,
    settings: &Settings,
) -> (bool, u32, usize, usize)
where
    T: Clone,
{
    assert!(content.len() >= 2);
    let initial_content = content.get_vec().clone();
    let old_len = initial_content.len();

    let chosen_indexes = prepare_double_indexes_to_remove(content.get_vec(), thread_rng);

    let mut iterations = 0;
    for (start_idx, end_idx) in chosen_indexes {
        iterations += 1;
        *content.get_mut_vec() = content.get_vec()[start_idx..end_idx].to_vec();
        let (is_broken, _output) = check_if_is_broken(content, &settings);
        if is_broken {
            return (true, iterations, old_len, content.len());
        }
        *content.get_mut_vec() = initial_content.clone();
    }
    (false, iterations, old_len, content.len())
}

fn remove_some_content_from_start_end<T>(
    content: &mut dyn DataTraits<T>,
    thread_rng: &mut ThreadRng,
    settings: &Settings,
    from_start: bool,
) -> (bool, u32, usize, usize)
where
    T: Clone,
{
    assert_ne!(content.len(), 0);
    let initial_content = content.get_vec().clone();
    let old_len = initial_content.len();

    let chosen_indexes = prepare_indexes_to_remove(content.get_vec(), thread_rng, from_start);

    let mut iterations = 0;
    for idx in chosen_indexes {
        iterations += 1;
        if from_start {
            *content.get_mut_vec() = content.get_vec()[idx..].to_vec();
        } else {
            *content.get_mut_vec() = content.get_vec()[..idx].to_vec();
        }
        let (is_broken, _output) = check_if_is_broken(content, &settings);
        if is_broken {
            return (true, iterations, old_len, content.len());
        }
        *content.get_mut_vec() = initial_content.clone();
    }
    (false, iterations, old_len, content.len())
}

// Prepares indexes in group of 2
// Indexes are sorted and second value is always greater than first
// Indexes are unique
// Indexes are sorted by difference between them - at start we are checking if we can remove big chunk which should be more effective
fn prepare_double_indexes_to_remove<T>(
    content: &Vec<T>,
    thread_rng: &mut ThreadRng,
) -> Vec<(usize, usize)> {
    // Max 10 indexes to remove - no need to test more
    let indexes_to_remove = min(10, content.len().isqrt());
    let mut chosen_indexes: Vec<_> = (0..indexes_to_remove)
        .map(|_| (thread_rng.gen_range(0..content.len()), thread_rng.gen_range(0..content.len())))
        .map(|(a,b)| {
            if a > b {
                (b, a)
            } else {
                (a, b)
            }
        })
        .filter(|(a, b)| a != b)
        .collect();

    chosen_indexes.sort_by(|(a1,b1), (a2,b2)| {
        (b2-a2).cmp(&(b1-a1))
    });
    chosen_indexes.dedup();
    chosen_indexes
}

fn prepare_indexes_to_remove<T>(
    content: &Vec<T>,
    thread_rng: &mut ThreadRng,
    from_start: bool,
) -> Vec<usize> {
    let start_idx = if from_start { 1 } else { 0 };
    let end_idx = if from_start { content.len() } else { content.len() - 1 };
    // Max 10 indexes to remove - no need to test more
    let indexes_to_remove = min(10, content.len().isqrt());
    let mut chosen_indexes: Vec<_> = (0..indexes_to_remove)
        .map(|_| thread_rng.gen_range(start_idx..end_idx))
        .collect();
    chosen_indexes.sort_unstable();
    chosen_indexes.dedup();
    if from_start {
        chosen_indexes.reverse();
    }
    chosen_indexes
}

fn create_command(settings: &Settings, file_name: &str) -> String {
    settings
        .command
        .replace("{}", &format!("\"{}\"", file_name))
}

fn check_if_is_broken<T>(content: &dyn DataTraits<T>, settings: &Settings) -> (bool, String) {
    if let Err(e) = content.save_to_file(&settings.output_file) {
        eprintln!("Error writing file {}, reason {}", &settings.output_file, e);
        process::exit(1);
    }
    let command = create_command(&settings, &settings.output_file);
    let output = process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    let all = collect_output(&output);
    (all.contains(&settings.broken_info), all)
}

pub fn collect_output(output: &Output) -> String {
    let stdout = &output.stdout;
    let stderr = &output.stderr;
    let stdout_str = String::from_utf8_lossy(stdout);
    let stderr_str = String::from_utf8_lossy(stderr);
    let status_signal = format!(
        "====== Status {:?}, Signal {:?}",
        output.status.code(),
        output.status.signal()
    );
    format!("{stdout_str}\n{stderr_str}\n\n{status_signal}")
}

fn load_content(settings: &Settings) -> Vec<u8> {
    if !Path::new(&settings.input_file).exists() {
        eprintln!("File {} does not exists", &settings.input_file);
        process::exit(1);
    }
    let content = match fs::read(&settings.input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}, reason {}", &settings.input_file, e);
            process::exit(1);
        }
    };

    if settings.character_mode {
        if String::from_utf8(content.clone()).is_err() {
            eprintln!("File {} is not ascii file", &settings.input_file);
            process::exit(1);
        }
    }

    if let Err(e) = fs::write(&settings.output_file, &content) {
        eprintln!("Error writing file {}, reason {}", &settings.output_file, e);
        process::exit(1);
    }

    if let Err(e) = fs::remove_file(&settings.output_file) {
        eprintln!(
            "Error removing file {}, reason {}",
            &settings.output_file, e
        );
        process::exit(1);
    }

    content
}
