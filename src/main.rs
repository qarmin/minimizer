#![feature(isqrt)]

use clap::Parser;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::os::unix::prelude::ExitStatusExt;
use std::path::Path;
use std::process::Output;
use std::{fs, process};
use strum_macros::Display;

#[derive(Parser)]
#[command(name = "Files minimizator")]
#[command(author = "Rafał Mikrut")]
#[command(version = "1.0.0")]
#[command(
    about = "Minimize files",
    long_about = "App that minimizes files, to find smallest possible file that have."
)]
pub struct Settings {
    #[arg(
        short,
        long,
        value_name = "INPUT",
        help = "Input file that will be minimized"
    )]
    pub(crate) input_file: String,

    #[arg(
        short,
        long,
        value_name = "OUTPUT",
        help = "Output file to save results"
    )]
    pub(crate) output_file: String,

    #[arg(short, long, value_name = "NUMBER", help = "Attempts to minimize file")]
    pub(crate) attempts: u32,

    #[arg(
        short,
        long,
        value_name = "NUMBER",
        help = "Reset attempts counter to start value, when file was minimized in current iteration",
        default_value_t = 100
    )]
    pub(crate) reset_attempts: u32,

    #[arg(
        short = 'u',
        long,
        value_name = "IS_CHARACTER_MODE",
        help = "Operates on characters instead of bytes, will fail if file is not text file",
        default_value_t = false
    )]
    pub(crate) character_mode: bool,

    #[arg(
        short = 's',
        long,
        value_name = "IS_ASCII_MODE",
        help = "Operates on ascii characters instead of bytes/utf-8, will fail if file not contains only ascii characters",
        default_value_t = false
    )]
    pub(crate) ascii_mode: bool,

    #[arg(
        short,
        long,
        value_name = "COMMAND",
        help = "Command which will be used to minimize e.g. 'godot {} -c 1000'\nBy default {} is used as placeholder for file, but this can be changed.\nAll occurences of \" will be replaced with '"
    )]
    pub(crate) command: String,

    #[arg(
        short,
        long,
        value_name = "BROKEN_CONTENT",
        help = "Content inside output of command, that will show that file is broken",
        default_value = "{}"
    )]
    pub(crate) broken_info: String,
}

trait DataTraits<T> {
    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error>;
    fn get_mut_vec(&mut self) -> &mut Vec<T>;
    fn get_vec(&self) -> &Vec<T>;
    fn len(&self) -> usize {
        self.get_vec().len()
    }
}
pub struct MinimizationBytes {
    pub(crate) bytes: Vec<u8>,
}
impl DataTraits<u8> for MinimizationBytes {
    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error> {
        fs::write(file_name, &self.bytes)
    }
    fn get_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
    fn get_vec(&self) -> &Vec<u8> {
        &self.bytes
    }
}
pub struct MinimizationLines {
    pub(crate) lines: Vec<String>,
}
impl DataTraits<String> for MinimizationLines {
    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error> {
        fs::write(file_name, self.lines.join("\n"))
    }
    fn get_mut_vec(&mut self) -> &mut Vec<String> {
        &mut self.lines
    }
    fn get_vec(&self) -> &Vec<String> {
        &self.lines
    }
}
pub struct MinimizationChars {
    pub(crate) chars: Vec<char>,
}
impl DataTraits<char> for MinimizationChars {
    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error> {
        fs::write(file_name, &self.chars.iter().collect::<String>())
    }
    fn get_mut_vec(&mut self) -> &mut Vec<char> {
        &mut self.chars
    }
    fn get_vec(&self) -> &Vec<char> {
        &self.chars
    }
}

#[derive(Display)]
pub enum Mode {
    #[strum(serialize = "bytes")]
    Bytes,
    #[strum(serialize = "lines")]
    Lines,
    #[strum(serialize = "chars")]
    Chars,
}

fn main() {
    let mut settings = Settings::parse();
    settings.command = settings.command.replace("\"", "'");

    println!("Example command: {}", create_command(&settings, "test.jpg"));

    let initial_file_content = load_content(&settings);

    println!("Initial file size: {}", initial_file_content.len());

    let mut minimization_bytes = MinimizationBytes {
        bytes: initial_file_content.clone(),
    };

    let (is_initially_broken, initial_output) = check_if_is_broken(&minimization_bytes, &settings);
    if !is_initially_broken {
        eprintln!("File is not broken, check command or file");
        eprintln!("==================OUTPUT==================");
        eprintln!("{initial_output}");
        eprintln!("===========================================");
        process::exit(1);
    }

    let mut thread_rng = rand::thread_rng();
    let mut iterations_counter = 0;

    extend_results(
        remove_some_content_from_start_end(
            &mut minimization_bytes,
            &mut thread_rng,
            &settings,
            true,
        ),
        &mut iterations_counter,
        Mode::Bytes,
    );
    extend_results(
        remove_some_content_from_start_end(
            &mut minimization_bytes,
            &mut thread_rng,
            &settings,
            false,
        ),
        &mut iterations_counter,
        Mode::Bytes,
    );
}

fn extend_results(result: (bool, u32, usize, usize), iterations_counter: &mut u32, mode: Mode) {
    let (changed, iterations, old_len, new_len) = result;
    *iterations_counter += iterations;
    if changed {
        println!("File was changed from {} to {} {}", old_len, new_len, mode);
    }
}

fn remove_some_content_from_start_end<T>(
    content: &mut dyn DataTraits<T>,
    thread_rng: &mut ThreadRng,
    settings: &Settings,
    from_start: bool,
) -> (bool, u32, usize, usize)
where
    T: Clone,
{
    assert_ne!(content.len(), 0);
    let initial_content = content.get_vec().clone();
    let old_len = initial_content.len();

    let chosen_indexes = prepare_indexes_to_remove(content.get_vec(), thread_rng);

    let mut iterations = 0;
    for idx in chosen_indexes {
        iterations += 1;
        if from_start {
            *content.get_mut_vec() = content.get_vec()[idx..].to_vec();
        } else {
            *content.get_mut_vec() = content.get_vec()[..idx].to_vec();
        }
        let (is_broken, _output) = check_if_is_broken(content, &settings);
        if is_broken {
            return (true, iterations, old_len, content.len());
        }
    }

    // Not broken, restore initial content
    *content.get_mut_vec() = initial_content;

    (false, iterations, old_len, content.len())
}

fn prepare_indexes_to_remove<T>(content: &Vec<T>, thread_rng: &mut ThreadRng) -> Vec<usize> {
    let indexes_to_remove = content.len().isqrt();
    let mut chosen_indexes: Vec<_> = (0..indexes_to_remove)
        .map(|_| thread_rng.gen_range(0..content.len()))
        .collect();
    chosen_indexes.sort_unstable();
    chosen_indexes.dedup();
    chosen_indexes
}

fn create_command(settings: &Settings, file_name: &str) -> String {
    settings
        .command
        .replace("{}", &format!("\"{}\"", file_name))
}

fn check_if_is_broken<T>(content: &dyn DataTraits<T>, settings: &Settings) -> (bool, String) {
    if let Err(e) = content.save_to_file(&settings.output_file) {
        eprintln!("Error writing file {}, reason {}", &settings.output_file, e);
        process::exit(1);
    }
    let command = create_command(&settings, &settings.output_file);
    let output = process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    let all = collect_output(&output);
    (all.contains(&settings.broken_info), all)
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

fn load_content(settings: &Settings) -> Vec<u8> {
    if Path::new(&settings.input_file).exists() {
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

    match (settings.character_mode, settings.ascii_mode) {
        (false, true) => {
            // Second condition is probably not needed
            if !content.iter().all(|&c| c.is_ascii()) || String::from_utf8(content.clone()).is_err()
            {
                eprintln!("File {} is not ascii file", &settings.input_file);
                process::exit(1);
            }
        }
        (true, false) => {
            if String::from_utf8(content.clone()).is_err() {
                eprintln!("File {} is not ascii file", &settings.input_file);
                process::exit(1);
            }
        }
        _ => {}
    };

    if let Err(e) = fs::write(&settings.output_file, &content) {
        eprintln!("Error writing file {}, reason {}", &settings.output_file, e);
        process::exit(1);
    }
    if let Err(e) = fs::remove_file(&settings.output_file) {
        eprintln!(
            "Error removing file {}, reason {}",
            &settings.output_file, e
        );
        process::exit(1);
    }

    content
}
