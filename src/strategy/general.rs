use std::fmt::Debug;

use rand::prelude::ThreadRng;

use crate::data_trait::{DataTraits, SaveSliceToFile};
use crate::rules::{Rule, RuleType};
use crate::settings::Settings;
use crate::strategy::common::{
    check_if_stopping_minimization, execute_rule_and_extend_results, execute_rules_until_first_found_broken,
    ProcessStatus, Strategy,
};
use crate::Stats;

// Simple strategy that should work for most cases
// How this works:
// - Tries to remove elements from start and end(quite easy way to remove a lot of elements)
// - Randomly removes elements from the middle
// - If less than 5 elements left, tries to remove all combinations of 2, 3, 4 elements
pub struct GeneralStrategy<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> GeneralStrategy<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T> Strategy<T> for GeneralStrategy<T>
where
    T: Clone + SaveSliceToFile + Send + Sync + Debug,
{
    fn minimize(&self, stats: &mut Stats, settings: &Settings, mm: &mut dyn DataTraits<T>, rng: &mut ThreadRng) {
        minimize_internal(stats, settings, mm, rng);

        // After minimization to less than 5 elements, we try to remove all combinations of 2, 3, 4 elements

        if mm.len() <= 4 && mm.len() >= 2 {
            let all_combination_rules = Rule::create_all_combinations_rule(mm.len());
            let _ = execute_rules_until_first_found_broken(all_combination_rules, stats, settings, mm, false);
        }
    }
}

fn minimize_internal<T>(stats: &mut Stats, settings: &Settings, mm: &mut dyn DataTraits<T>, _rng: &mut ThreadRng)
where
    T: Clone + SaveSliceToFile + Send + Sync + Debug,
{
    const REMOVE_FROM_START_ITERS: usize = 5;
    const REMOVE_FROM_END_ITERS: usize = 35;

    for (iters, from_start) in [(REMOVE_FROM_START_ITERS, true), (REMOVE_FROM_END_ITERS, false)] {
        if check_if_stopping_minimization(stats, settings, mm.get_vec(), true) == ProcessStatus::Stop {
            return;
        }

        let from_start_end_rules = Rule::create_start_end_rule(mm.len(), iters, from_start);

        if execute_rules_until_first_found_broken(from_start_end_rules, stats, settings, mm, true)
            == ProcessStatus::Stop
        {
            return;
        };
    }

    loop {
        if check_if_stopping_minimization(stats, settings, mm.get_vec(), true) == ProcessStatus::Stop {
            return;
        }

        let _ = execute_rule_and_extend_results(&get_random_rule(mm.len()), stats, settings, mm);
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
