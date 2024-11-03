use std::time::Instant;
use std::{fs, process};

use clap::Parser;
use once_cell::sync::Lazy;
use rand::prelude::ThreadRng;

use crate::common::{check_if_is_broken, create_command, load_and_check_files};
use crate::data_trait::{DataTraits, MinimizationBytes, MinimizationChars, MinimizationLines, Mode};
use crate::settings::Settings;
use crate::strategy::common::Strategy;
use crate::strategy::general::{GeneralStrategy};

mod common;
mod data_trait;
mod rules;
mod settings;
mod strategy;

pub static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

#[derive(Default)]
pub struct Stats {
    pub(crate) all_iterations: u32,
    pub(crate) current_iteration_count: u32,
    pub(crate) max_attempts: u32,
}
impl Stats {
    pub fn new() -> Self {
        Stats::default()
    }
    pub fn available(&self) -> u32 {
        self.max_attempts - self.current_iteration_count
    }
    pub fn increase(&mut self, how_much: u32) {
        self.all_iterations += how_much;
        self.current_iteration_count += how_much;
    }
    pub fn reset(&mut self) {
        self.current_iteration_count = 0;
    }
}

fn main() {
    let _ = *START_TIME; // To initialize lazy static

    let mut settings = Settings::parse();
    settings.command = settings.command.replace("\"", "'");

    let start_time = Instant::now();
    let initial_file_content = load_and_check_files(&settings);

    if settings.is_normal_message_visible() {
        println!(
            "Starting to test file \"{}\" - Initial file size: {} bytes, with command: \n{}\nList of searched strings: {:?}\nList of ignored strings: {:?}",
            settings.input_file,
            initial_file_content.len(),
            create_command(&settings),
            settings.broken_info,
            settings.ignored_info
        );
    }

    let mb = MinimizationBytes {
        bytes: initial_file_content.clone(),
        mode: Mode::Bytes,
    };
    let (is_initially_broken, initial_output) = check_if_is_broken(&mb, &settings);

    if !is_initially_broken {
        eprintln!("File is not broken, check command or file");
        eprintln!("==================COMMAND=================");
        eprintln!("{}", create_command(&settings));
        eprintln!("==================OUTPUT==================");
        eprintln!("{initial_output}");
        eprintln!("===========================================");
        process::exit(1);
    }

    if let Err(e) = fs::copy(&settings.input_file, &settings.output_file) {
        eprintln!("Error copying file {}, reason {}", &settings.output_file, e);
        process::exit(1);
    }

    let mut rng = rand::thread_rng();
    let mut stats = Stats {
        all_iterations: 0,
        current_iteration_count: 0,
        max_attempts: 0,
    };

    let mb = minimize_content(initial_file_content.clone(), &mut stats, &settings, &mut rng);

    if !check_if_is_broken(&mb, &settings).0 && settings.is_normal_message_visible() {
        eprintln!("Minimized file was broken at start, but now is not - this may be bug in minimizer or app have not stable output.");
        eprintln!("==================COMMAND=================");
        eprintln!("{}", create_command(&settings));
        eprintln!("==================OUTPUT==================");
        eprintln!("{initial_output}");
        eprintln!("==================CONTENT=================");
        eprintln!("{}", String::from_utf8_lossy(&mb.bytes));
        eprintln!("===========================================");
    }

    let bytes = mb.len();
    match mb.save_to_file(&settings.output_file) {
        Ok(_) => {
            if settings.is_normal_message_visible() {
                if bytes == initial_file_content.len() {
                    println!(
                        "File {} was not minimized, after {} iterations (limit was {}, retrying - {}) in {:?}",
                        &settings.output_file,
                        stats.all_iterations,
                        settings.attempts,
                        settings.reset_attempts,
                        start_time.elapsed()
                    );
                } else {
                    let initial_size_percent = (bytes as f64 / initial_file_content.len() as f64) * 100.0;
                    println!(
                        "File {} was minimized from {} to {} bytes({:.1}% of initial size), after {} iterations (limit was {}, retrying - {}) in {:?}",
                        &settings.output_file, initial_file_content.len(), bytes, initial_size_percent, stats.all_iterations, settings.attempts, settings.reset_attempts, start_time.elapsed()
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("Error writing file {}, reason {}", &settings.output_file, e);
            process::exit(1);
        }
    }
}

fn minimize_content(
    initial_file_content: Vec<u8>,
    stats: &mut Stats,
    settings: &Settings,
    rng: &mut ThreadRng,
) -> MinimizationBytes {
    let mut mb;
    if let Ok(initial_str_content) = String::from_utf8(initial_file_content.clone()) {
        let mut ms = MinimizationLines {
            mode: Mode::Lines,
            lines: initial_str_content.split("\n").map(|x| x.to_string()).collect(),
        };
        stats.max_attempts = settings.attempts / 3;
        GeneralStrategy::minimize(stats, settings, &mut ms, rng);

        let mut mc = MinimizationChars {
            mode: Mode::Chars,
            chars: ms.lines.join("\n").chars().collect(),
        };
        stats.max_attempts = settings.attempts * 2 / 3;
        GeneralStrategy::minimize(stats, settings, &mut mc, rng);

        mb = MinimizationBytes {
            mode: Mode::Bytes,
            bytes: mc.chars.iter().collect::<String>().as_bytes().to_vec(),
        };
    } else {
        mb = MinimizationBytes {
            mode: Mode::Bytes,
            bytes: initial_file_content.clone(),
        };
    }

    stats.max_attempts = settings.attempts;
    GeneralStrategy::minimize(stats, settings, &mut mb, rng);

    mb
}
