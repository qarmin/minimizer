use std::fmt::Debug;
use std::os::unix::prelude::ExitStatusExt;
use std::path::Path;
use std::process::{Output, Stdio};
use std::{fs, process};

use crate::data_trait::SaveSliceToFile;
use crate::settings::{get_temp_file, Settings};

pub fn create_command(settings: &Settings) -> String {
    let base_command = create_single_command_str(settings, &get_temp_file(), &settings.command);
    if let Some(additional_command) = &settings.additional_command {
        let new_command = create_single_command_str(settings, &get_temp_file(), additional_command);
        format!("{base_command}; {new_command}")
    } else {
        base_command
    }
}

fn create_single_command_str(settings: &Settings, file_name: &str, input_command: &str) -> String {
    if settings.disable_file_name_escaping {
        input_command.replace(&settings.file_symbol, file_name)
    } else {
        input_command.replace(&settings.file_symbol, &format!("\"{}\"", file_name))
    }
}

pub fn check_if_is_broken<T>(content: &[T], settings: &Settings) -> (bool, String)
where
    T: Clone + SaveSliceToFile + Send + Sync + Debug,
{
    if let Err(e) = T::save_slice_to_file(content, &get_temp_file()) {
        eprintln!("Error writing file {}, reason {}", &get_temp_file(), e);
        process::exit(1);
    }
    let command = create_command(settings);

    // TODO split into 2 different commands
    let start_time = std::time::Instant::now();
    let output = process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    let elapsed = start_time.elapsed();
    let all = collect_output(&output);

    let contains_broken_info = settings.broken_info.iter().any(|info| all.contains(info));
    let contains_ignored_info = settings
        .ignored_info
        .as_ref()
        .map_or(false, |ignored| ignored.iter().any(|info| all.contains(info)));

    let is_broken = contains_broken_info && !contains_ignored_info;

    if settings.print_command_output && settings.is_normal_message_visible() {
        println!(
            "=========================\n{}\nMinimization result - contains broken info \"{}\", contains ignored info \"{}\", is broken \"{}\", took {elapsed:?}\n=========================",
            all, contains_broken_info, contains_ignored_info, is_broken
        );
    }

    (is_broken, all)
}

pub fn collect_output(output: &Output) -> String {
    let stdout = &output.stdout;
    let stderr = &output.stderr;
    let stdout_str = String::from_utf8_lossy(stdout);
    let stderr_str = String::from_utf8_lossy(stderr);
    let status_signal = format!(
        "====== Status {:?}, Signal {:?}",
        output.status.code(),
        output.status.signal()
    );
    format!("{stdout_str}\n{stderr_str}\n\n{status_signal}")
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
