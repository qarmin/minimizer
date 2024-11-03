use std::fs;

use strum_macros::Display;

pub trait DataTraits<T: Clone> {
    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error>;
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
    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error> {
        fs::write(file_name, &self.bytes)
    }
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
    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error> {
        fs::write(file_name, self.lines.join("\n"))
    }
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
    fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error> {
        fs::write(file_name, self.chars.iter().collect::<String>())
    }
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

#[derive(Display, Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    #[strum(serialize = "bytes")]
    Bytes,
    #[strum(serialize = "lines")]
    Lines,
    #[strum(serialize = "chars")]
    Chars,
}
