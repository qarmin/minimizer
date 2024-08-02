use crate::data_trait::DataTraits;
use crate::settings::Settings;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::cmp::{max, min};
use std::os::unix::prelude::ExitStatusExt;
use std::process;
use std::process::{Output, Stdio};

pub fn create_command(settings: &Settings, file_name: &str) -> String {
    if settings.disable_file_name_escaping {
        settings.command.replace(&settings.file_symbol, file_name)
    } else {
        settings
            .command
            .replace(&settings.file_symbol, &format!("\"{}\"", file_name))
    }
}

pub fn check_if_is_broken<T>(content: &dyn DataTraits<T>, settings: &Settings) -> (bool, String) {
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
    let contains_broken_info = settings.broken_info.iter().any(|info| all.contains(info));
    let contains_ignored_info = settings
        .ignored_info
        .as_ref()
        .map_or(false, |ignored| ignored.iter().any(|info| all.contains(info)));
    (contains_broken_info && !contains_ignored_info, all)
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

// Prepares indexes in group of 2
// Indexes are sorted and second value is always greater than first
// Indexes are unique
// Indexes are sorted by difference between them - at start we are checking if we can remove big chunk which should be more effective
pub fn prepare_double_indexes_to_remove<T>(
    content: &Vec<T>,
    thread_rng: &mut ThreadRng,
    max_iterations: usize,
) -> Vec<(usize, usize)> {
    let indexes_to_remove = max(min(max_iterations, content.len().isqrt()), 1);
    let mut chosen_indexes: Vec<_> = (0..indexes_to_remove)
        .map(|_| {
            (
                thread_rng.gen_range(0..content.len()),
                thread_rng.gen_range(0..content.len()),
            )
        })
        .map(|(a, b)| if a > b { (b, a) } else { (a, b) })
        .filter(|(a, b)| a != b)
        .collect();

    chosen_indexes.sort_by(|(a1, b1), (a2, b2)| (b2 - a2).cmp(&(b1 - a1)));
    chosen_indexes.dedup();
    chosen_indexes
}

pub fn prepare_indexes_to_remove<T>(
    content: &Vec<T>,
    thread_rng: &mut ThreadRng,
    max_iterations: usize,
    from_start: bool,
) -> Vec<usize> {
    let start_idx = if from_start { 1 } else { 0 };
    let end_idx = if from_start { content.len() } else { content.len() - 1 };
    let indexes_to_remove = max(min(max_iterations, content.len().isqrt()), 1);

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

pub fn prepare_random_indexes_to_remove<T>(
    content: &Vec<T>,
    thread_rng: &mut ThreadRng,
    max_iterations: usize,
) -> Vec<Vec<usize>> {
    let stats = max(min(max_iterations, content.len().isqrt()), 1);
    let mut chosen_indexes = vec![];

    for _ in 0..stats {
        let mut current_indexes = vec![];
        for _ in 0..=thread_rng.gen_range(1..=stats) {
            current_indexes.push(thread_rng.gen_range(0..content.len()));
        }
        current_indexes.sort_unstable();
        current_indexes.dedup();
        chosen_indexes.push(current_indexes);
    }

    chosen_indexes.sort_unstable();
    chosen_indexes.dedup();
    chosen_indexes
}
