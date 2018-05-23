extern crate clap;
#[macro_use]
extern crate failure;
extern crate goblin;
extern crate parity_wasm;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate xfailure;

mod config;
mod errors;
mod functions_ids;
mod functions_names;
mod map;
mod patcher;
mod sections;
mod symbols;

use config::*;
use errors::*;
use patcher::*;

fn main() -> Result<(), WError> {
    let config = Config::parse_cmdline()?;
    let patcher = Patcher::from_file(config.patcher_config, config.input_path)?;
    patcher.store_to_file(config.output_path)?;
    Ok(())
}
