use clap::Parser;

#[derive(Parser)]
#[command(name = "minimizer")]
#[command(author = "Rafał Mikrut")]
#[command(version = "1.0.0")]
#[command(
    about = "Minimize files",
    long_about = "App that minimizes files, to find the smallest possible file that have certain output."
)]
pub struct Settings {
    #[arg(short, long, value_name = "INPUT", help = "Input file that will be minimized")]
    pub(crate) input_file: String,

    #[arg(short, long, value_name = "OUTPUT", help = "Output file to save results")]
    pub(crate) output_file: String,

    #[arg(short, long, value_name = "NUMBER", help = "Attempts to minimize file")]
    pub(crate) attempts: u32,

    #[arg(
        short,
        long,
        value_name = "NUMBER",
        help = "Reset attempts counter to start value, when file was minimized in current iteration",
        default_value_t = false
    )]
    pub(crate) reset_attempts: bool,

    #[arg(
        short,
        long,
        value_name = "COMMAND",
        help = "Command which will be used to minimize e.g. 'godot {} -c 1000'\nBy default {} is used as placeholder for file, but this can be changed.\nAll occurrences of \" will be replaced with '"
    )]
    pub(crate) command: String,

    #[arg(
        short,
        long,
        value_name = "SYMBOL",
        help = "Symbol that will be replaced with file name in command, by default {}",
        default_value = "{}"
    )]
    pub(crate) file_symbol: String,

    #[arg(
        short,
        long,
        value_name = "DISABLE_ESCAPING",
        help = "Removes \"\" from file name, when passing replacing file symbol from command.\nBy default 'cargo {}' will be converted to \'cargo \"/home/user/some path.jpg\"'\nWith this flag it will be converted to 'cargo /home/user/some path.jpg' so you need to escape spaces in file name manually",
        default_value_t = false
    )]
    pub(crate) disable_file_name_escaping: bool,

    #[arg(
        short,
        long,
        num_args = 1..,
        value_name = "BROKEN_CONTENT",
        help = "Content inside output of command, that will show that file is broken"
    )]
    pub(crate) broken_info: Vec<String>,

    #[arg(
        short = 'z',
        long,
        value_name = "IGNORED_CONTENT",
        help = "Content inside output of command, that will be ignored"
    )]
    pub(crate) ignored_info: Option<Vec<String>>,
}
