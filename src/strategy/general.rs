use crate::data_trait::{DataTraits, Mode};
use crate::rules::{create_remove_all_combinations_rule, create_remove_continuous_rule, create_remove_from_start_end_rules, create_remove_random_indexes_rule, execute_rule, Rule, RULE_TYPE};
use crate::settings::Settings;
use crate::{Stats, START_TIME};
use rand::distributions::WeightedIndex;
use rand::prelude::ThreadRng;
use rand::Rng;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ProcessStatus {
    Continue,
    Stop,
}

fn execute_rules_until_first_found_broken<T>(
    rules: Vec<Rule>,
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    mode: Mode,
    check_length: bool,
) -> ProcessStatus {
    for rule in rules {
        if check_if_stopping_minimization(stats, settings, mm, check_length) == ProcessStatus::Stop {
            return ProcessStatus::Stop;
        }

        if execute_rule_and_extend_results(rule, stats, settings, mm, mode) {
            return ProcessStatus::Continue;
        }
    }
    ProcessStatus::Continue
}

#[must_use]
fn execute_rule_and_extend_results<T>(
    rule: Rule,
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    mode: Mode,
) -> bool {
    let old_len = mm.len();
    let is_broken = rule.execute(mm, settings);
    extend_results(is_broken, 1, old_len, mm.len(), stats, mode, settings);
    is_broken
}

fn minimize_general_internal<T>(
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    mode: Mode,
    _rng: &mut ThreadRng,
) {
    const REMOVE_FROM_START_ITERS: usize = 5;
    const REMOVE_FROM_MIDDLE_CONST_ITERS: usize = 20;
    const REMOVE_FROM_END_ITERS: usize = 35;
    const REMOVE_FROM_MIDDLE_RANDOM_ITERS: usize = 20;
    const REMOVE_FROM_START_END_ITERS: usize = 2;
    const ONE_BY_ONE_LIMIT: usize = 100;

    for (iters, from_start) in [(REMOVE_FROM_START_ITERS, true), (REMOVE_FROM_START_ITERS, false)] {
        let from_end_rules = Rule::create_start_end_rule(mm.len(), iters, from_start);
        if execute_rules_until_first_found_broken(from_end_rules, stats, settings, mm, mode, true) == ProcessStatus::Stop {
            return;
        };
    }

    // Normal rules
    loop {
        if check_if_stopping_minimization(stats, settings, mm, true) == ProcessStatus::Stop {
            return;
        }
        execute_rule_and_extend_results(get_random_rule(mm.len()), stats, settings, mm, mode);
    }
}


pub fn get_random_rule(content_size: usize) -> Rule {
    let rules_weights = [
        (RULE_TYPE::RemoveFromStart, 2),
        (RULE_TYPE::RemoveFromEnd, 10),
        (RULE_TYPE::RemoveContinuousFromMiddle, 30),
        (RULE_TYPE::RemoveRandom, 10),
    ];
    let weights = rules_weights.iter().map(|(_, w)| *w).collect::<Vec<_>>();
    let mut dist = WeightedIndex::new(&weights).expect("Cannot fail because we provide weights");
    let chosen = rules_weights[dist.sample(&mut rand::thread_rng())];

    match chosen {
        RULE_TYPE::RemoveFromStart => Rule::create_start_end_rule(content_size, 1, true)[0].clone(),

        RULE_TYPE::RemoveFromEnd => Rule::create_start_end_rule(content_size, 1, false)[0].clone(),
        RULE_TYPE::RemoveContinuousFromMiddle => Rule::create_continuous_rule(content_size),
        RULE_TYPE::RemoveRandom => Rule::create_random_rule(content_size, None),
    }
}


pub fn minimize_general<T>(
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    mode: Mode,
    rng: &mut ThreadRng,
) {
    minimize_general_internal(stats, settings, mm, mode, rng);

    // After minimization to less than 5 elements, we try to remove all combinations of 2, 3, 4 elements

    if mm.len() <= 4 {
        if mm.len() >= 2 {
            let all_combination_rules = Rule::create_all_combinations_rule(mm.len());
            let _ = execute_rules_until_first_found_broken(all_combination_rules, stats, settings, mm, mode, false);
        }
    }
}

#[must_use]
fn check_if_stopping_minimization<T>(
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    check_length: bool,
) -> ProcessStatus {
    if check_if_exceeded_time(settings) == ProcessStatus::Stop {
        return ProcessStatus::Stop;
    }

    if check_if_exceeded_iterations(stats) == ProcessStatus::Stop {
        return ProcessStatus::Stop;
    }

    if check_length {
        if mm.len() <= 4 {
            return ProcessStatus::Stop;
        }
    }

    ProcessStatus::Continue
}


fn check_if_exceeded_time(settings: &Settings) -> ProcessStatus {
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

fn check_if_exceeded_iterations(stats: &Stats) -> ProcessStatus {
    if stats.available() == 0 {
        return ProcessStatus::Stop;
    }
    ProcessStatus::Continue
}

fn minimize_general<T>(
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
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
            extend_results(changed, iterations, old_len, mm.len(), stats, mm.get_mode(), settings);
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
