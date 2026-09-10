use std::{error::Error, path::Path};

use crate::config::Config;

pub fn run(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let config = Config::load(config_path)?;
    println!("checking version {}...", config.version);

    Ok(())
}
