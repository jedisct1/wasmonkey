mod config;

use wasmonkey::*;

use crate::config::*;

fn main() -> Result<(), Error> {
    let config = Config::parse_cmdline()?;
    let patcher = Patcher::from_file(config.patcher_config, config.input_path)?;
    patcher.store_to_file(config.output_path)?;
    Ok(())
}
