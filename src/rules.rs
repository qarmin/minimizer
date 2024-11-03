use rand::{thread_rng, Rng};

use crate::common::check_if_is_broken;
use crate::data_trait::DataTraits;
use crate::settings::Settings;

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
impl Rule {
    pub fn create_start_end_rule(
        content_size: usize,
        number_of_checks: usize,
        remove_from_start: bool,
    ) -> Vec<Rule> {
        assert!(content_size >= 5);

        let mut chosen_indexes = if content_size < number_of_checks {
            (1..content_size).into_iter().collect()
        } else {
            (0..content_size)
                .into_iter()
                .map(|_| thread_rng().gen_range(1..content_size))
                .collect::<Vec<_>>()
        };
        if remove_from_start {
            chosen_indexes.sort_unstable();
            chosen_indexes.retain(|&x| (x != 0) && (x != content_size - 1)); // We don't want to remove entire content
            chosen_indexes
                .into_iter()
                .map(|remove_to_idx| Rule::RemoveContinuous {
                    start_idx_included: 0,
                    end_idx_excluded: remove_to_idx,
                })
                .collect()
        } else {
            chosen_indexes.sort_unstable();
            chosen_indexes.reverse();
            chosen_indexes.retain(|&x| (x != 0) && (x != content_size - 1)); // We don't want to remove entire content
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
        let max_number_of_indexes = number_of_indexes.unwrap_or_else(|| ((content_size as f32).sqrt() as usize).clamp(2, 100));

        let number_of_indexes = thread_rng().gen_range(2..max_number_of_indexes);
        let indexes_list = (1..(number_of_indexes.min(content_size - 1)))
            .into_iter()
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
    pub fn execute<T>(&self,content: &mut dyn DataTraits<T>, settings: &Settings) -> bool
    where
        T: Clone,
    {
        let initial_content = content.get_vec().clone();
        match &self {
            Rule::RemoveContinuous {
                start_idx_included,
                end_idx_excluded,
            } => {
                content.get_mut_vec().drain(start_idx_included..end_idx_excluded);
            }
            Rule::RemoveRandom { indexes_to_remove } => {
                content
                    .get_mut_vec()
                    .retain(|(idx, _)| !indexes_to_remove.contains(idx));
            }
        }

        let (is_broken, _output) = check_if_is_broken(&content, settings);

        if is_broken {
            *content.get_mut_vec() = initial_content;
        }

        is_broken
    }
}


