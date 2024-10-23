use crate::common::{check_if_is_broken, create_command};
use crate::data_trait::{DataTraits, MinimizationBytes, MinimizationChars, MinimizationLines, Mode};
use crate::rules::{
    load_and_check_files, minimize_smaller_than_5_lines, remove_certain_continous_indexes, remove_certain_idx,
    remove_continuous_content_from_middle, remove_random_content_from_middle, remove_some_content_from_start_end,
};
use crate::settings::Settings;
use clap::Parser;
use rand::prelude::ThreadRng;
use std::time::Instant;
use std::{fs, process};

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
    const REMOVE_FROM_START_END_ITERS: usize = 2;
    const ONE_BY_ONE_LIMIT: usize = 100;

    if settings.is_verbose_message_visible() {
        println!(
            "Using {mode} mode ({}/{} attempts({} max in current mode, {} all iterations))",
            stats.current_iteration_count, settings.attempts, stats.max_attempts, stats.all_iterations
        );
    }

    if !minimize_smaller_and_or_exit(mm, settings, stats) {
        return;
    }

    if stats.available() >= 30 * mm.len() as u32 {
        println!("Using special mode to remove content from end");
        for idx in 1..(mm.len() - 1) {
            let old_len = mm.len();
            let (changed, iterations) = remove_certain_continous_indexes(mm, settings, idx, old_len - 1);
            extend_results(changed, iterations, old_len, mm.len(), stats, mode, settings);
            if changed {
                break; // We cannot minimize this more, because we remove content from end
            }
        }
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

    // This step is designed to remove some chunks from middle
    // Random checking won't check all this possibilities so this should be more effective
    if stats.available() >= 50 * mm.len() as u32 {
        println!("Using special mode to remove content from middle");
        // offset allows to start from different position, it is done mostly for 2 step
        // to be able to check entire file with overlapped 2 byte windows
        'outside: for i in [150, 50, 10, 5, 2] {
            for off in [0, 1] {
                let start_len = ((mm.len() - off) % i) + i;
                if start_len >= mm.len() {
                    continue;
                }
                for j in (start_len..mm.len()).step_by(i).rev() {
                    let start_idx = j - i;
                    let end_idx = j;
                    let old_len = mm.len();
                    let (changed, iterations) = remove_certain_continous_indexes(mm, settings, start_idx, end_idx);
                    extend_results(changed, iterations, old_len, mm.len(), stats, mode, settings);
                    if mm.len() <= 4 {
                        break 'outside;
                    }
                }
            }
        }
        if !minimize_smaller_and_or_exit(mm, settings, stats) {
            eprintln!("Step with minimizing in windows, should not ends entire checking");
            return;
        }
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
            let (changed, iterations) =
                remove_some_content_from_start_end(mm, rng, settings, REMOVE_FROM_START_END_ITERS, stats, from_start);
            extend_results(changed, iterations, old_len, mm.len(), stats, mode, settings);
        }

        if !minimized_one_by_one && mm.len() <= ONE_BY_ONE_LIMIT && stats.available() > mm.len() as u32 {
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
