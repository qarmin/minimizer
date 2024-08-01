#![feature(isqrt)]

use std::os::unix::prelude::ExitStatusExt;
use std::path::Path;
use std::{fs, process};

use clap::Parser;
use rand::prelude::ThreadRng;
use rand::Rng;

use crate::common::{check_if_is_broken, create_command, prepare_double_indexes_to_remove, prepare_indexes_to_remove};
use crate::data_trait::{DataTraits, MinimizationBytes, Mode};
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
        if mb.len() >= 2 {
            extend_results(
                remove_continuous_content_from_middle(&mut mb, &mut rng, &settings),
                &mut iters,
                Mode::Bytes,
            );
        }

        if mb.len() == 0 {
            break;
        }
        extend_results(
            remove_some_content_from_start_end(&mut mb, &mut rng, &settings, true),
            &mut iters,
            Mode::Bytes,
        );
        if mb.len() == 0 {
            break;
        }
        extend_results(
            remove_some_content_from_start_end(&mut mb, &mut rng, &settings, false),
            &mut iters,
            Mode::Bytes,
        );
        if mb.len() == 0 {
            break;
        }
    }

    match mb.save_to_file(&settings.output_file) {
        Ok(_) => {
            println!(
                "File was minimized to {}, after {} iterations",
                &settings.output_file, iters
            );
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

