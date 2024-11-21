use std::{fs, io};

use strum_macros::Display;

pub trait DataTraits<T: Clone> {
    fn get_vec(&self) -> &Vec<T>;
    fn replace_vec(&mut self, new_vec: Vec<T>);
    fn len(&self) -> usize {
        self.get_vec().len()
    }
    fn get_mode(&self) -> Mode;
}
pub struct MinimizationBytes {
    pub(crate) mode: Mode,
    pub(crate) bytes: Vec<u8>,
}
impl DataTraits<u8> for MinimizationBytes {
    fn get_vec(&self) -> &Vec<u8> {
        &self.bytes
    }
    fn replace_vec(&mut self, new_vec: Vec<u8>) {
        self.bytes = new_vec;
    }
    fn get_mode(&self) -> Mode {
        self.mode
    }
}
pub struct MinimizationLines {
    pub(crate) mode: Mode,
    pub(crate) lines: Vec<String>,
}
impl DataTraits<String> for MinimizationLines {
    fn get_vec(&self) -> &Vec<String> {
        &self.lines
    }
    fn replace_vec(&mut self, new_vec: Vec<String>) {
        self.lines = new_vec;
    }
    fn get_mode(&self) -> Mode {
        self.mode
    }
}
pub struct MinimizationChars {
    pub(crate) mode: Mode,
    pub(crate) chars: Vec<char>,
}
impl DataTraits<char> for MinimizationChars {
    fn get_vec(&self) -> &Vec<char> {
        &self.chars
    }
    fn replace_vec(&mut self, new_vec: Vec<char>) {
        self.chars = new_vec;
    }
    fn get_mode(&self) -> Mode {
        self.mode
    }
}

pub trait SaveSliceToFile {
    fn save_slice_to_file(slice: &[Self], file_name: &str) -> io::Result<()>
    where
        Self: Sized;
}

impl SaveSliceToFile for u8 {
    fn save_slice_to_file(slice: &[u8], file_name: &str) -> io::Result<()> {
        fs::write(file_name, slice)
    }
}

impl SaveSliceToFile for char {
    fn save_slice_to_file(slice: &[char], file_name: &str) -> io::Result<()> {
        fs::write(file_name, slice.iter().collect::<String>())
    }
}

impl SaveSliceToFile for String {
    fn save_slice_to_file(slice: &[String], file_name: &str) -> io::Result<()> {
        fs::write(file_name, slice.join("\n"))
    }
}

#[derive(Display, Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    #[strum(serialize = "bytes")]
    Bytes,
    #[strum(serialize = "lines")]
    Lines,
    #[strum(serialize = "chars")]
    Chars,
}
