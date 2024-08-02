use clap::Parser;

#[derive(Parser)]
#[command(name = "Files minimizator")]
#[command(author = "Rafał Mikrut")]
#[command(version = "1.0.0")]
#[command(
    about = "Minimize files",
    long_about = "App that minimizes files, to find smallest possible file that have."
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
        value_name = "BROKEN_CONTENT",
        help = "Content inside output of command, that will show that file is broken"
    )]
    pub(crate) broken_info: String,
}
