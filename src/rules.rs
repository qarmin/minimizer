use std::fmt::{Display, Formatter};

use rand::distributions::{Distribution, WeightedIndex};
use rand::{thread_rng, Rng};
use strum_macros::EnumIter;

use crate::common::check_if_is_broken;
use crate::data_trait::{Mode, SaveSliceToFile};
use crate::settings::Settings;
use crate::Stats;

#[allow(clippy::enum_variant_names)]
#[derive(EnumIter, Copy, Clone, Debug)]
pub enum RuleType {
    RemoveFromStart,
    RemoveFromEnd,
    RemoveContinuousFromMiddle,
    RemoveRandom,
}
impl RuleType {
    // Function will panic if not provided weights
    // This is responsibility of caller to provide correct weights
    pub fn get_random_type(weights: &[(RuleType, usize)]) -> RuleType {
        let dist = WeightedIndex::new(weights.iter().map(|(_, weight)| *weight)).expect("Not provided weights");

        weights[dist.sample(&mut thread_rng())].0
    }
}

#[derive(Clone, Debug)]
pub enum Rule {
    RemoveContinuous {
        start_idx_included: usize,
        end_idx_excluded: usize,
    },
    RemoveRandom {
        indexes_to_remove: Vec<usize>,
    },
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Rule::RemoveContinuous {
                start_idx_included,
                end_idx_excluded,
            } => write!(f, "RemoveContinuous: {}..{}", start_idx_included, end_idx_excluded),
            Rule::RemoveRandom { indexes_to_remove } => {
                write!(f, "RemoveRandom: {:?}", indexes_to_remove)
            }
        }
    }
}

impl Rule {
    pub fn create_start_end_rule(content_size: usize, number_of_checks: usize, remove_from_start: bool) -> Vec<Rule> {
        assert!(content_size >= 5);

        let mut chosen_indexes = if content_size < number_of_checks {
            // When number of requested checks is bigger than content size, we will collect all possible ranges
            // For 5 element, we collect here 1, 2, 3, 4, so ranges are 0..1(1 element), 0..2(2 element), 0..3(3 elements), 0..4(4 elements)
            (1..content_size).collect()
        } else {
            // When number of requested checks is smaller than content size, we will collect random ranges
            (0..number_of_checks)
                .map(|_| thread_rng().gen_range(1..content_size))
                .collect::<Vec<_>>()
        };
        chosen_indexes.sort_unstable();
        chosen_indexes.dedup();

        if remove_from_start {
            chosen_indexes.reverse();
            chosen_indexes
                .into_iter()
                .map(|remove_to_idx| Rule::RemoveContinuous {
                    start_idx_included: 0,
                    end_idx_excluded: remove_to_idx,
                })
                .collect()
        } else {
            chosen_indexes
                .into_iter()
                .map(|remove_from_idx| Rule::RemoveContinuous {
                    start_idx_included: remove_from_idx,
                    end_idx_excluded: content_size,
                })
                .collect()
        }
    }

    pub fn create_continuous_rule(content_size: usize) -> Rule {
        assert!(content_size >= 5);
        let start_idx = thread_rng().gen_range(1..content_size - 2);
        let end_idx = thread_rng().gen_range(start_idx + 1..content_size - 1);
        Rule::RemoveContinuous {
            start_idx_included: start_idx,
            end_idx_excluded: end_idx,
        }
    }

    pub fn create_random_rule(content_size: usize, number_of_indexes: Option<usize>) -> Rule {
        assert!(content_size >= 5);
        if let Some(number_of_indexes) = number_of_indexes {
            assert!(number_of_indexes >= 2);
        }
        let max_number_of_indexes =
            number_of_indexes.unwrap_or_else(|| ((content_size as f32).sqrt() as usize).clamp(3, 100));

        let number_of_indexes = thread_rng().gen_range(2..max_number_of_indexes);
        let indexes_list = (1..(number_of_indexes.min(content_size - 1)))
            .map(|_| thread_rng().gen_range(0..content_size))
            .collect();
        Rule::RemoveRandom {
            indexes_to_remove: indexes_list,
        }
    }

    pub fn create_all_combinations_rule(content_size: usize) -> Vec<Rule> {
        assert!((2..5).contains(&content_size));

        let all_combinations: &[&[usize]] = match content_size {
            2 => &[&[0], &[1]],
            3 => &[&[0, 1], &[0, 2], &[1, 2], &[0], &[1], &[2]],
            4 => &[
                &[0, 1, 2],
                &[0, 1, 3],
                &[0, 2, 3],
                &[1, 2, 3],
                &[0, 1],
                &[0, 2],
                &[0, 3],
                &[1, 2],
                &[1, 3],
                &[2, 3],
                &[0],
                &[1],
                &[2],
                &[3],
            ],
            _ => unreachable!(),
        };

        all_combinations
            .iter()
            .map(|indexes_to_remove| Rule::RemoveRandom {
                indexes_to_remove: indexes_to_remove.to_vec(),
            })
            .collect()
    }
    pub fn crete_remove_exact_idx_rule(idxs: Vec<usize>) -> Rule {
        Rule::RemoveRandom {
            indexes_to_remove: idxs,
        }
    }
    pub fn execute<T>(&self, stats: &Stats, content: &[T], mode: Mode, settings: &Settings) -> Option<Vec<T>>
    where
        T: Clone + SaveSliceToFile + Send + Sync,
    {
        assert!(!content.is_empty());
        if settings.is_extra_verbose_message_visible() {
            println!(
                "Executing rule: {} ({} iteration), with size: {} {}, ",
                self,
                stats.all_iterations + 1,
                content.len(),
                mode
            );
        }
        let mut test_content = content.to_vec();
        match &self {
            Rule::RemoveContinuous {
                start_idx_included,
                end_idx_excluded,
            } => {
                test_content.drain(start_idx_included..end_idx_excluded);
            }
            Rule::RemoveRandom { indexes_to_remove } => {
                let new_vec = content
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| !indexes_to_remove.contains(idx))
                    .map(|(_, x)| x.clone())
                    .collect();
                test_content = new_vec;
            }
        }

        let (is_broken, _output) = check_if_is_broken(&test_content, settings);

        if is_broken {
            Some(test_content)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_type_get_random_type() {
        let weights = [(RuleType::RemoveFromStart, 2), (RuleType::RemoveFromEnd, 10)];
        let mut remove_from_start = 0;
        let mut remove_from_end = 0;
        for _ in 0..100 {
            match RuleType::get_random_type(&weights) {
                RuleType::RemoveFromStart => remove_from_start += 1,
                RuleType::RemoveFromEnd => remove_from_end += 1,
                _ => unreachable!(),
            }
        }
        assert_eq!(remove_from_start + remove_from_end, 100);
    }

    #[test]
    fn test_rule_create_start_end_rule() {
        let content_size = 10;
        let number_of_checks = 100;
        let remove_from_start = true;
        let rules = Rule::create_start_end_rule(content_size, number_of_checks, remove_from_start);
        assert!(!rules.is_empty()); // We can't predict exact number of checks due deduplication, but it should be more than 0

        let mut previous_diff = usize::MAX;
        for rule in rules {
            match rule {
                Rule::RemoveContinuous {
                    start_idx_included,
                    end_idx_excluded,
                } => {
                    assert!(start_idx_included < end_idx_excluded);
                    assert_eq!(start_idx_included, 0);
                    assert!(end_idx_excluded < content_size);

                    // We start checking from the biggest range, to get at start the biggest minimization
                    let diff = end_idx_excluded - start_idx_included;
                    assert!(diff < previous_diff);
                    previous_diff = diff;
                }
                _ => unreachable!(),
            }
        }

        let remove_from_start = false;
        let rules = Rule::create_start_end_rule(content_size, number_of_checks, remove_from_start);
        assert!(!rules.is_empty()); // We can't predict exact number of checks due deduplication, but it should be more than 0

        let mut previous_diff = usize::MAX;
        for rule in rules {
            match rule {
                Rule::RemoveContinuous {
                    start_idx_included,
                    end_idx_excluded,
                } => {
                    assert!(start_idx_included < end_idx_excluded);
                    assert!(start_idx_included > 0);
                    assert_eq!(end_idx_excluded, content_size);

                    // We start checking from the biggest range, to get at start the biggest minimization
                    let diff = end_idx_excluded - start_idx_included;
                    assert!(diff < previous_diff);
                    previous_diff = diff;
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_rule_create_continuous_rule() {
        for _ in 0..100 {
            let content_size = 10;
            let rule = Rule::create_continuous_rule(content_size);
            match rule {
                Rule::RemoveContinuous {
                    start_idx_included,
                    end_idx_excluded,
                } => {
                    assert!(start_idx_included < end_idx_excluded);
                    assert!(start_idx_included > 0);
                    assert!(end_idx_excluded < content_size);
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_rule_create_random_rule() {
        for _ in 0..100 {
            let content_size = 10;
            let rule = Rule::create_random_rule(content_size, None);
            match rule {
                Rule::RemoveRandom { indexes_to_remove } => {
                    assert!(!indexes_to_remove.is_empty());
                    for idx in &indexes_to_remove {
                        assert!(*idx < content_size);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
