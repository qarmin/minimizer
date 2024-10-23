use crate::common::{
    check_if_is_broken, prepare_double_indexes_to_remove, prepare_indexes_to_remove, prepare_random_indexes_to_remove,
};
use crate::data_trait::DataTraits;
use crate::settings::{get_temp_file, Settings};
use crate::Stats;
use rand::prelude::ThreadRng;
use std::path::Path;
use std::{fs, mem, process};

pub fn remove_random_content_from_middle<T>(
    content: &mut dyn DataTraits<T>,
    thread_rng: &mut ThreadRng,
    settings: &Settings,
    max_tool_iterations: usize,
    stats: &Stats,
) -> (bool, u32)
where
    T: Clone,
{
    assert!(content.len() >= 5);
    let initial_content = content.get_vec().clone();

    let chosen_indexes = prepare_random_indexes_to_remove(content.get_vec(), thread_rng, max_tool_iterations, stats);

    let mut iterations_used = 0;
    for indexes_to_remove in chosen_indexes {
        iterations_used += 1;
        *content.get_mut_vec() = content
            .get_vec()
            .iter()
            .enumerate()
            .filter(|(idx, _)| !indexes_to_remove.contains(idx))
            .map(|(_, v)| v)
            .cloned()
            .collect();
        let (is_broken, _output) = check_if_is_broken(content, settings);
        if is_broken {
            return (true, iterations_used);
        }
        *content.get_mut_vec() = initial_content.clone();
    }
    (false, iterations_used)
}

pub fn remove_continuous_content_from_middle<T>(
    content: &mut dyn DataTraits<T>,
    thread_rng: &mut ThreadRng,
    settings: &Settings,
    max_tool_iterations: usize,
    stats: &Stats,
) -> (bool, u32)
where
    T: Clone,
{
    assert!(content.len() >= 5);
    let initial_content = content.get_vec().clone();

    let chosen_indexes = prepare_double_indexes_to_remove(content.get_vec(), thread_rng, max_tool_iterations, stats);

    let mut iterations_used = 0;
    for (start_idx, end_idx) in chosen_indexes {
        iterations_used += 1;
        let mut new_vec = content.get_vec()[..start_idx].to_vec();
        new_vec.extend_from_slice(&content.get_vec()[end_idx..]);
        *content.get_mut_vec() = new_vec;
        let (is_broken, _output) = check_if_is_broken(content, settings);
        if is_broken {
            return (true, iterations_used);
        }
        *content.get_mut_vec() = initial_content.clone();
    }
    (false, iterations_used)
}

pub fn remove_certain_idx<T>(content: &mut dyn DataTraits<T>, settings: &Settings, idx: usize) -> (bool, u32)
where
    T: Clone,
{
    assert!(idx < content.len());
    let initial_content = content.get_vec().clone();

    let mut new_content = mem::take(content.get_mut_vec());
    new_content.remove(idx);

    *content.get_mut_vec() = new_content;
    let (is_broken, _output) = check_if_is_broken(content, settings);
    if is_broken {
        return (true, 1);
    }
    *content.get_mut_vec() = initial_content.clone();
    (false, 1)
}
pub fn remove_certain_continous_indexes<T>(
    content: &mut dyn DataTraits<T>,
    settings: &Settings,
    start_idx: usize,
    end_idx: usize,
) -> (bool, u32)
where
    T: Clone,
{
    assert!(start_idx < content.len());
    assert!(end_idx < content.len());
    let initial_content = content.get_vec().clone();

    let mut new_content = mem::take(content.get_mut_vec());
    new_content.drain(start_idx..end_idx);

    *content.get_mut_vec() = new_content;
    let (is_broken, _output) = check_if_is_broken(content, settings);
    if is_broken {
        return (true, 1);
    }
    *content.get_mut_vec() = initial_content.clone();
    (false, 1)
}

pub fn remove_some_content_from_start_end<T>(
    content: &mut dyn DataTraits<T>,
    thread_rng: &mut ThreadRng,
    settings: &Settings,
    max_tool_iterations: usize,
    stats: &Stats,
    from_start: bool,
) -> (bool, u32)
where
    T: Clone,
{
    assert!(content.len() >= 5);
    let initial_content = content.get_vec().clone();
    let chosen_indexes =
        prepare_indexes_to_remove(content.get_vec(), thread_rng, max_tool_iterations, stats, from_start);

    let mut iterations_used = 0;
    for idx in chosen_indexes {
        iterations_used += 1;
        if from_start {
            *content.get_mut_vec() = content.get_vec()[idx..].to_vec();
        } else {
            *content.get_mut_vec() = content.get_vec()[..idx].to_vec();
        }
        let (is_broken, _output) = check_if_is_broken(content, settings);
        if is_broken {
            return (true, iterations_used);
        }
        *content.get_mut_vec() = initial_content.clone();
    }
    (false, iterations_used)
}

// When lines is 2-4, we can calculate all possible combinations and check them
// It is the only mode that not takes into account number of iterations and can exceed it
// If app have stable results, this function should create the smallest possible file
pub fn minimize_smaller_than_5_lines<T>(content: &mut dyn DataTraits<T>, settings: &Settings) -> bool
where
    T: Clone,
{
    assert!(content.len() >= 2 && content.len() <= 4);
    let initial_content = content.get_vec().clone();

    let all_combinations: &[&[usize]] = match content.len() {
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

    for indexes_to_remove in all_combinations {
        *content.get_mut_vec() = content
            .get_vec()
            .iter()
            .enumerate()
            .filter(|(idx, _)| !indexes_to_remove.contains(idx))
            .map(|(_, v)| v)
            .cloned()
            .collect();
        let (is_broken, _output) = check_if_is_broken(content, settings);
        if is_broken {
            return true;
        }
        *content.get_mut_vec() = initial_content.clone();
    }

    false
}

pub fn load_and_check_files(settings: &Settings) -> Vec<u8> {
    if !Path::new(&settings.input_file).exists() {
        eprintln!("File {} does not exists", &settings.input_file);
        process::exit(1);
    }
    let content = match fs::read(&settings.input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}, reason {}", &settings.input_file, e);
            process::exit(1);
        }
    };

    for file in [get_temp_file(), settings.output_file.clone()] {
        if let Err(e) = fs::write(&file, &content) {
            eprintln!("Error writing file {}, reason {}", &file, e);
            process::exit(1);
        }

        if let Err(e) = fs::remove_file(&file) {
            eprintln!("Error removing file {}, reason {}", &file, e);
            process::exit(1);
        }
    }

    content
}
