use clap::{App, Arg};
use errors::*;
use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct Config {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub builtins_path: PathBuf,
}

impl Config {
    pub fn parse_cmdline() -> Result<Self, WError> {
        let matches = App::new("wasmonkey")
            .version("1.0")
            .about("Transforms WASM exports to imports")
            .arg(
                Arg::with_name("input_file")
                    .short("i")
                    .long("input")
                    .takes_value(true)
                    .required(true)
                    .help("Path to the input file"),
            )
            .arg(
                Arg::with_name("output_file")
                    .short("o")
                    .long("output")
                    .takes_value(true)
                    .required(true)
                    .help("Path to the output file"),
            )
            .arg(
                Arg::with_name("builtins_file")
                    .short("b")
                    .long("builtins")
                    .takes_value(true)
                    .required(true)
                    .help("Path to the builtins library"),
            )
            .get_matches();
        let input_path = PathBuf::from(matches
            .value_of("input_file")
            .ok_or(WError::UsageError("Input file required"))?);
        let output_path = PathBuf::from(matches
            .value_of("output_file")
            .ok_or(WError::UsageError("Output file required"))?);
        let builtins_path = PathBuf::from(matches
            .value_of("builtins_file")
            .ok_or(WError::UsageError("Builtins file required"))?);
        let config = Config {
            input_path,
            output_path,
            builtins_path,
        };
        Ok(config)
    }
}
