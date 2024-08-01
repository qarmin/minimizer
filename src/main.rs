#![feature(isqrt)]

use std::process;

use clap::Parser;
use rand::prelude::ThreadRng;

use crate::common::{check_if_is_broken, create_command};
use crate::data_trait::{DataTraits, MinimizationBytes, MinimizationChars, MinimizationLines, Mode};
use crate::rules::{load_content, remove_continuous_content_from_middle, remove_some_content_from_start_end};
use crate::settings::Settings;

mod common;
mod data_trait;
mod rules;
mod settings;

fn main() {
    let mut settings = Settings::parse();
    settings.command = settings.command.replace("\"", "'");

    println!("Example command: {}", create_command(&settings, "test.jpg"));

    let initial_file_content = load_content(&settings);

    println!("Initial file size: {}", initial_file_content.len());

    let mb = MinimizationBytes {
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

    let mut mb;
    if let Ok(initial_str_content) = String::from_utf8(initial_file_content.clone()) {
        let mut ms = MinimizationLines {
            lines: initial_str_content.split("\n").map(|x| x.to_string()).collect(),
        };
        minimize_general(
            &mut iters,
            &settings,
            settings.attempts / 3,
            &mut ms,
            Mode::Lines,
            &mut rng,
        );

        let mut mc = MinimizationChars {
            chars: ms.lines.join("\n").chars().collect(),
        };
        minimize_general(
            &mut iters,
            &settings,
            settings.attempts * 2 / 3,
            &mut mc,
            Mode::Chars,
            &mut rng,
        );

        mb = MinimizationBytes {
            bytes: mc.chars.iter().collect::<String>().as_bytes().to_vec(),
        };
    } else {
        mb = MinimizationBytes {
            bytes: initial_file_content.clone(),
        };
    }

    minimize_general(&mut iters, &settings, settings.attempts, &mut mb, Mode::Bytes, &mut rng);

    let bytes = mb.len();
    match mb.save_to_file(&settings.output_file) {
        Ok(_) => {
            println!(
                "File {} was minimized to {} bytes, after {} iterations",
                &settings.output_file, bytes, iters
            );
        }
        Err(e) => {
            eprintln!("Error writing file {}, reason {}", &settings.output_file, e);
            process::exit(1);
        }
    }
}

fn minimize_general<T>(
    iters: &mut u32,
    settings: &Settings,
    max_attempts: u32,
    mm: &mut dyn DataTraits<T>,
    mode: Mode,
    rng: &mut ThreadRng,
) where
    T: Clone,
{
    println!("Using {mode} mode");

    // At start, we can try to remove big chunks from start/end - inside loop later, this is probably not effective
    for from_start in [false, true] {
        if mm.len() < 2 || *iters >= max_attempts {
            return;
        }
        let old_len = mm.len();
        let (changed, iterations, new_len) = remove_some_content_from_start_end(mm, rng, settings, 20, from_start);
        extend_results(changed, iterations, old_len, new_len, iters, mode);
    }

    loop {
        for from_start in [false, true] {
            if mm.len() < 2 || *iters >= max_attempts {
                break;
            }
            let old_len = mm.len();
            let (changed, iterations, new_len) = remove_some_content_from_start_end(mm, rng, settings, 3, from_start);
            extend_results(changed, iterations, old_len, new_len, iters, mode);
        }

        if mm.len() < 2 || *iters >= max_attempts {
            break;
        }
        let old_len = mm.len();
        let (changed, iterations, new_len) = remove_continuous_content_from_middle(mm, rng, settings, 20);
        extend_results(changed, iterations, old_len, new_len, iters, mode);
    }
}

fn extend_results(
    changed: bool,
    iterations: u32,
    old_len: usize,
    new_len: usize,
    iterations_counter: &mut u32,
    mode: Mode,
) {
    *iterations_counter += iterations;
    if changed {
        println!("File was changed from {} to {} {}", old_len, new_len, mode);
    }
}
