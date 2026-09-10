use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use minijinja::{Environment, context};

use crate::config::Config;

const TEMPLATE: &str = include_str!("../../templates/build.html");

#[derive(Args)]
pub struct BuildArgs {
    /// Path for the generated HTML file.
    #[arg(short, long, default_value = "everarc.html", value_name = "PATH")]
    pub output: PathBuf,
}

pub fn run(config_path: &Path, args: BuildArgs) -> Result<(), Box<dyn Error>> {
    let config = Config::load(config_path)?;
    let mut environment = Environment::new();
    environment.add_template("build.html", TEMPLATE)?;
    let template = environment.get_template("build.html")?;
    let html = template.render(context!(version => config.version))?;

    fs::write(args.output, html)?;
    Ok(())
}
