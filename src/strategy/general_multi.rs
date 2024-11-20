use crate::data_trait::{DataTraits, SaveSliceToFile};
use crate::rules::{Rule, RuleType};
use crate::settings::Settings;
use crate::strategy::common::{
    check_if_stopping_minimization, execute_rule_and_extend_results,
    ProcessStatus, Strategy,
};
use crate::Stats;
use once_cell::sync::Lazy;
use rand::prelude::ThreadRng;
use rayon::prelude::*;
use std::num::NonZeroUsize;
use std::thread::available_parallelism;

pub static NUMBER_OF_THREADS: Lazy<usize> = Lazy::new(|| {
    usize::from(available_parallelism().unwrap_or(NonZeroUsize::new(1).expect("Cannot fail")))
});

// Simple strategy that should work for most cases
// How this works:
// - Tries to remove elements from start and end(quite easy way to remove a lot of elements)
// - Randomly removes elements from the middle
// - If less than 5 elements left, tries to remove all combinations of 2, 3, 4 elements
pub struct GeneralMultiStrategy<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> GeneralMultiStrategy<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T> Strategy<T> for GeneralMultiStrategy<T>
where
    T: Clone + SaveSliceToFile
{
    fn minimize(&self, stats: &mut Stats, settings: &Settings, mm: &mut dyn DataTraits<T>, rng: &mut ThreadRng) {
        minimize_internal(stats, settings, mm, rng);

        // After minimization to less than 5 elements, we try to remove all combinations of 2, 3, 4 elements

        if mm.len() <= 4 && mm.len() >= 2 {
            let all_combination_rules = Rule::create_all_combinations_rule(mm.len());
            let _ = execute_multi_rules_until_first_found_broken(all_combination_rules, stats, settings, mm, false);
        }
    }
}

pub(crate) fn execute_multi_rules_until_first_found_broken<T>(
    rules: Vec<Rule>,
    stats: &mut Stats,
    settings: &Settings,
    mm: &mut dyn DataTraits<T>,
    check_length: bool,
) -> ProcessStatus
where
    T: Clone + SaveSliceToFile
{
    // let available_stats = stats.available();
    // let old_len = mm.len();
    //
    // let results = rules.into_par_iter().take(available_stats as usize).filter_map(|rule|
    //     {
    //         let cloned_mm = mm.clone();
    //         if check_if_stopping_minimization(stats, settings, cloned_mm, check_length) == ProcessStatus::Stop {
    //             return None;
    //         }
    //         let is_broken = rule.execute(stats, cloned_mm, settings);
    //
    //         if is_broken {
    //             Some((Some(cloned_mm), 1))
    //         } else {
    //             Some((None, 1))
    //         }
    //     }).while_some().flatten().collect::<Vec<_>>();


    // for rule in rules {
    //     if check_if_stopping_minimization(stats, settings, mm.get_vec(), check_length) == ProcessStatus::Stop {
    //         return ProcessStatus::Stop;
    //     }
    //     if execute_rule_and_extend_results(rule, stats, settings, mm) {
    //         return ProcessStatus::Continue;
    //     }
    // }
    ProcessStatus::Continue
}


fn minimize_internal<T>(stats: &mut Stats, settings: &Settings, mm: &mut dyn DataTraits<T>, _rng: &mut ThreadRng)
where
    T: Clone + SaveSliceToFile
{
    const REMOVE_FROM_START_ITERS: usize = 5;
    const REMOVE_FROM_END_ITERS: usize = 35;

    for (iters, from_start) in [(REMOVE_FROM_START_ITERS, true), (REMOVE_FROM_END_ITERS, false)] {
        if check_if_stopping_minimization(stats, settings, mm.get_vec(), true) == ProcessStatus::Stop {
            return;
        }

        let from_start_end_rules = Rule::create_start_end_rule(mm.len(), iters, from_start);

        if execute_multi_rules_until_first_found_broken(from_start_end_rules, stats, settings, mm, true)
            == ProcessStatus::Stop
        {
            return;
        };
    }

    loop {
        if check_if_stopping_minimization(stats, settings, mm.get_vec(), true) == ProcessStatus::Stop {
            return;
        }

        let _ = execute_rule_and_extend_results(get_random_rule(mm.len()), stats, settings, mm);
    }
}

pub fn get_random_rule(content_size: usize) -> Rule {
    let rules_weights = [
        (RuleType::RemoveFromStart, 2),
        (RuleType::RemoveFromEnd, 10),
        (RuleType::RemoveContinuousFromMiddle, 30),
        (RuleType::RemoveRandom, 10),
    ];
    let chosen = RuleType::get_random_type(&rules_weights);

    match chosen {
        RuleType::RemoveFromStart => Rule::create_start_end_rule(content_size, 1, true)[0].clone(),
        RuleType::RemoveFromEnd => Rule::create_start_end_rule(content_size, 1, false)[0].clone(),
        RuleType::RemoveContinuousFromMiddle => Rule::create_continuous_rule(content_size),
        RuleType::RemoveRandom => Rule::create_random_rule(content_size, None),
    }
}
