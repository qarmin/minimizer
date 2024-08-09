#![feature(isqrt)]

use clap::Parser;
use rand::prelude::ThreadRng;
use std::process;
use std::time::Instant;

use crate::common::{check_if_is_broken, create_command};
use crate::data_trait::{DataTraits, MinimizationBytes, MinimizationChars, MinimizationLines, Mode};
use crate::rules::{
    load_content, minimize_smaller_than_5_lines, remove_certain_idx, remove_continuous_content_from_middle,
    remove_random_content_from_middle, remove_some_content_from_start_end,
};
use crate::settings::Settings;

mod common;
mod data_trait;
mod rules;
mod settings;

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
    pub fn increase(&mut self, how_much: u32) {
        self.all_iterations += how_much;
        self.current_iteration_count += how_much;
    }
    pub fn reset(&mut self) {
        self.current_iteration_count = 0;
    }
}

fn main() {
    let mut settings = Settings::parse();
    settings.command = settings.command.replace("\"", "'");

    let start_time = Instant::now();
    let initial_file_content = load_content(&settings);

    if settings.is_normal_message_visible() {
        println!(
            "Starting to test file \"{}\" - Initial file size: {} bytes, with command: {}",
            settings.input_file,
            initial_file_content.len(),
            create_command(&settings)
        );
    }

    let mb = MinimizationBytes {
        bytes: initial_file_content.clone(),
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

    let mut rng = rand::thread_rng();
    let mut stats = Stats {
        all_iterations: 0,
        current_iteration_count: 0,
        max_attempts: 0,
    };

    let mut mb;
    if let Ok(initial_str_content) = String::from_utf8(initial_file_content.clone()) {
        let mut ms = MinimizationLines {
            lines: initial_str_content.split("\n").map(|x| x.to_string()).collect(),
        };
        stats.max_attempts = settings.attempts / 3;
        minimize_general(&mut stats, &settings, &mut ms, Mode::Lines, &mut rng);

        let mut mc = MinimizationChars {
            chars: ms.lines.join("\n").chars().collect(),
        };
        stats.max_attempts = settings.attempts * 2 / 3;
        minimize_general(&mut stats, &settings, &mut mc, Mode::Chars, &mut rng);

        mb = MinimizationBytes {
            bytes: mc.chars.iter().collect::<String>().as_bytes().to_vec(),
        };
    } else {
        mb = MinimizationBytes {
            bytes: initial_file_content.clone(),
        };
    }

    stats.max_attempts = settings.attempts;
    minimize_general(&mut stats, &settings, &mut mb, Mode::Bytes, &mut rng);

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

fn minimize_general<T>(
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    mode: Mode,
    rng: &mut ThreadRng,
) where
    T: Clone,
{
    const REMOVE_FROM_START_ITERS: usize = 5;
    const REMOVE_FROM_MIDDLE_CONST_ITERS: usize = 20;
    const REMOVE_FROM_END_ITERS: usize = 35;
    const REMOVE_FROM_MIDDLE_RANDOM_ITERS: usize = 20;
    const ONE_BY_ONE_LIMIT: usize = 100;

    if settings.is_verbose_message_visible() {
        println!(
            "Using {mode} mode ({}/{} attempts({} max in current mode, {} all iterations))",
            stats.current_iteration_count, settings.attempts, stats.max_attempts, stats.all_iterations
        );
    }

    // At start, we can try to remove big chunks from start/end - inside loop later, this is probably not effective
    for from_start in [false, true] {
        let iterations = if from_start {
            REMOVE_FROM_START_ITERS
        } else {
            REMOVE_FROM_END_ITERS
        };

        if !minimize_smaller_and_or_exit(mm, settings, stats) {
            return;
        }

        let old_len = mm.len();
        let (changed, iterations) =
            remove_some_content_from_start_end(mm, rng, settings, iterations, stats, from_start);
        extend_results(changed, iterations, old_len, mm.len(), stats, mode, settings);
    }

    let mut minimized_one_by_one = false;

    'start: loop {
        if !minimize_smaller_and_or_exit(mm, settings, stats) {
            break 'start;
        }
        let old_len = mm.len();
        let (changed, iterations) =
            remove_continuous_content_from_middle(mm, rng, settings, REMOVE_FROM_MIDDLE_CONST_ITERS, stats);
        extend_results(changed, iterations, old_len, mm.len(), stats, mode, settings);

        if !minimize_smaller_and_or_exit(mm, settings, stats) {
            break 'start;
        }
        let old_len = mm.len();
        let (changed, iterations) =
            remove_random_content_from_middle(mm, rng, settings, REMOVE_FROM_MIDDLE_RANDOM_ITERS, stats);
        extend_results(changed, iterations, old_len, mm.len(), stats, mode, settings);

        for from_start in [false, true] {
            if !minimize_smaller_and_or_exit(mm, settings, stats) {
                break 'start;
            }
            let old_len = mm.len();
            let (changed, iterations) = remove_some_content_from_start_end(mm, rng, settings, 2, stats, from_start);
            extend_results(changed, iterations, old_len, mm.len(), stats, mode, settings);
        }

        let available_iterations = stats.max_attempts - stats.current_iteration_count;
        if !minimized_one_by_one && mm.len() <= ONE_BY_ONE_LIMIT && available_iterations > mm.len() as u32 {
            if !minimize_smaller_and_or_exit(mm, settings, stats) {
                break 'start;
            }
            minimized_one_by_one = true;
            for idx in (0..mm.len()).rev() {
                let old_len = mm.len();
                let (changed, iterations) = remove_certain_idx(mm, settings, idx);
                extend_results(changed, iterations, old_len, mm.len(), stats, mode, settings);
            }
        }
    }
}

fn minimize_smaller_and_or_exit<T>(mm: &mut dyn DataTraits<T>, settings: &Settings, stats: &Stats) -> bool
where
    T: Clone,
{
    if mm.len() <= 4 {
        if mm.len() >= 2 {
            minimize_smaller_than_5_lines(mm, settings);
        }
        return false;
    }
    if stats.current_iteration_count >= stats.max_attempts {
        return false;
    }
    true
}

fn extend_results(
    changed: bool,
    iterations: u32,
    old_len: usize,
    new_len: usize,
    stats: &mut Stats,
    mode: Mode,
    settings: &Settings,
) {
    stats.increase(iterations);
    if changed {
        assert_ne!(old_len, new_len);
        if settings.is_verbose_message_visible() {
            println!(
                "File was changed from {} to {} {} ({} attempt)",
                old_len, new_len, mode, stats.all_iterations
            );
        }
        if settings.reset_attempts {
            stats.reset();
        }
    }
}
