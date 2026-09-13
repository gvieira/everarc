use std::{error::Error, path::Path};

use crate::config::Config;

pub fn run(config_path: &Path) -> Result<(), Box<dyn Error>> {
    Config::load(config_path)?;
    println!("configuration is valid.");

    Ok(())
}
