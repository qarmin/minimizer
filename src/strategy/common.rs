use std::process;

use crate::data_trait::{DataTraits, Mode};
use crate::rules::Rule;
use crate::settings::Settings;
use crate::strategy::general::ProcessStatus;
use crate::{Stats, START_TIME};

pub(crate) fn extend_results(
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

pub(crate) fn check_if_exceeded_time(settings: &Settings) -> ProcessStatus {
    if let Some(max_time) = settings.max_time {
        if START_TIME.elapsed().as_secs() >= max_time as u64 {
            if settings.is_normal_message_visible() {
                println!("Max time exceeded, stopping minimization");
            }
            return ProcessStatus::Stop;
        }
    }
    ProcessStatus::Continue
}

pub(crate) fn check_if_exceeded_iterations(stats: &Stats) -> ProcessStatus {
    if stats.available() == 0 {
        return ProcessStatus::Stop;
    }
    ProcessStatus::Continue
}

#[must_use]
pub(crate) fn check_if_stopping_minimization<T>(
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    check_length: bool,
) -> ProcessStatus
where
    T: Clone,
{
    if check_if_exceeded_time(settings) == ProcessStatus::Stop {
        return ProcessStatus::Stop;
    }

    if check_if_exceeded_iterations(stats) == ProcessStatus::Stop {
        return ProcessStatus::Stop;
    }

    if check_length && mm.len() <= 4 {
        return ProcessStatus::Stop;
    }

    ProcessStatus::Continue
}

pub(crate) fn execute_rules_until_first_found_broken<T>(
    rules: Vec<Rule>,
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    check_length: bool,
) -> ProcessStatus
where
    T: Clone,
{
    for rule in rules {
        if check_if_stopping_minimization(stats, settings, mm, check_length) == ProcessStatus::Stop {
            return ProcessStatus::Stop;
        }

        if execute_rule_and_extend_results(rule, stats, settings, mm) {
            return ProcessStatus::Continue;
        }
    }
    ProcessStatus::Continue
}

#[must_use]
pub(crate) fn execute_rule_and_extend_results<T>(
    rule: Rule,
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
) -> bool
where
    T: Clone,
{
    let old_len = mm.len();
    let is_broken = rule.execute(mm, settings);
    extend_results(is_broken, 1, old_len, mm.len(), stats, mm.get_mode(), settings);

    // Also saves current minimal output to file, to be able to get results even if app will be stopped by user in the middle of minimization
    if is_broken {
        if let Err(e) = mm.save_to_file(&settings.output_file) {
            eprintln!("Error writing file {}, reason {}", &settings.output_file, e);
            process::exit(1);
        }
    }

    is_broken
}
