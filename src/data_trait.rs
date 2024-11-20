use std::{fs, io};
use std::io::Write;
use strum_macros::Display;

pub trait DataTraits<T: Clone> {
    fn get_mut_vec(&mut self) -> &mut Vec<T>;
    fn get_vec(&self) -> &Vec<T>;
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
    fn get_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
    fn get_vec(&self) -> &Vec<u8> {
        &self.bytes
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
    fn get_mut_vec(&mut self) -> &mut Vec<String> {
        &mut self.lines
    }
    fn get_vec(&self) -> &Vec<String> {
        &self.lines
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
    fn get_mut_vec(&mut self) -> &mut Vec<char> {
        &mut self.chars
    }
    fn get_vec(&self) -> &Vec<char> {
        &self.chars
    }
    fn get_mode(&self) -> Mode {
        self.mode
    }
}

pub trait SaveSliceToFile {
    fn save_slice_to_file(slice: &[Self], file_name: &str) -> io::Result<()> where Self: Sized;
}

impl SaveSliceToFile for u8 {
    fn save_slice_to_file(slice: &[u8], file_name: &str) -> io::Result<()> {
        fs::write(file_name, slice)
    }
}

impl SaveSliceToFile for char {
    fn save_slice_to_file(slice: &[char], file_name: &str) -> io::Result<()> {
        fs::write(file_name, &slice.iter().collect::<String>())
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
