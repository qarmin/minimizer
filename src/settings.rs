use clap::Parser;

thread_local! {
    pub static TEMP_FILE: String = format!("/tmp/minimizer_{}", std::process::id());
}

pub fn get_temp_file() -> String {
    TEMP_FILE.with(|f| f.clone())
}

#[derive(Parser)]
#[command(name = "minimizer")]
#[command(author = "Rafał Mikrut")]
#[command(version = "1.3.2")]
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

    #[arg(
        short,
        long,
        value_name = "QUIET",
        help = "Do not print any output, only errors",
        default_value_t = false
    )]
    quiet: bool,

    #[arg(
        short,
        long,
        value_name = "VERBOSE",
        help = "Prints more information",
        default_value_t = false
    )]
    verbose: bool,

    // #[arg(
    //     short="vv",
    //     long,
    //     value_name = "EXTRA_VERBOSE",
    //     help = "Prints command output when file is minimized, this may be useful, but will slow down minimization",
    //     default_value_t = false
    // )]
    // extra_verbose: bool,
    #[arg(
        short,
        long,
        value_name = "PRINT_OUTPUT",
        help = "Prints command output when file is minimized, this may be useful, but will slow down minimization",
        default_value_t = false
    )]
    pub(crate) print_command_output: bool,

    #[arg(
        short,
        long,
        value_name = "MAX_TIME_SECONDS",
        help = "Max time in seconds that minimization can take(this time will be exceeded, because minimizer needs to finish current iteration)"
    )]
    pub(crate) max_time: Option<u32>,

    #[arg(
        short = 'k',
        long,
        value_name = "ADDITIONAL_COMMAND",
        help = "Runs additional command, e.g. \"ruff {}\" can be command and \"python3 -m compileall {}\" additional command to verify that output file is valid(in any sense of this word)"
    )]
    pub(crate) additional_command: Option<String>,
}

impl Settings {
    pub fn is_normal_message_visible(&self) -> bool {
        !self.quiet
    }
    pub fn is_verbose_message_visible(&self) -> bool {
        !self.quiet && self.verbose
    }
    // pub fn is_verbose_message_visible(&self) -> bool {
    //     !self.quiet && (self.verbose || self.extra_verbose)
    // }
    // pub fn is_extra_verbose_message_visible(&self) -> bool {
    //     !self.quiet && self.extra_verbose
    // }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::settings::Settings;

    #[test]
    fn verify_cli() {
        Settings::command().debug_assert();
    }
}
