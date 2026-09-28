use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    time::{Duration, Instant},
};

use clap::Args;
use minijinja::{Environment, context};
use notify::{Event, RecursiveMode, Watcher};

use crate::{
    config::Config, data_export::ProjectionExport, localization::DashboardPresentation,
    projection::PlanProjection,
};

const TEMPLATE: &str = include_str!("../../templates/build.html");
const DASHBOARD_TEMPLATE: &str = include_str!("../../templates/components/dashboard.html");
const DEBOUNCE: Duration = Duration::from_millis(100);

#[derive(Args)]
pub struct BuildArgs {
    /// Path for the generated HTML file.
    #[arg(short, long, default_value = "everarc.html", value_name = "PATH")]
    pub output: PathBuf,

    /// Write the dashboard data model as JSON to this path.
    #[arg(long, value_name = "PATH")]
    pub data_output: Option<PathBuf>,

    /// Rebuild when the configuration file changes.
    #[arg(short, long)]
    pub watch: bool,
}

pub fn run(config_path: &Path, args: BuildArgs) -> Result<(), Box<dyn Error>> {
    let config_path = absolute_path(config_path)?;

    if args.watch {
        watch(&config_path, &args.output, args.data_output.as_deref())
    } else {
        build_once(&config_path, &args.output, args.data_output.as_deref())
    }
}

fn build_once(
    config_path: &Path,
    output_path: &Path,
    data_output_path: Option<&Path>,
) -> Result<(), Box<dyn Error>> {
    let config = Config::load(config_path)?;
    let projection = PlanProjection::from(&config);
    let dashboard =
        DashboardPresentation::with_actual_balances(&config, &projection, config.display.locale);
    fs::write(output_path, render_dashboard(&dashboard)?)?;
    if let Some(data_output_path) = data_output_path {
        let data = ProjectionExport::with_actual_balances(&config, &projection);
        fs::write(data_output_path, serde_json::to_string_pretty(&data)?)?;
    }
    Ok(())
}

fn render_dashboard(dashboard: &DashboardPresentation<'_>) -> Result<String, minijinja::Error> {
    let mut environment = Environment::new();
    environment.add_template("components/dashboard.html", DASHBOARD_TEMPLATE)?;
    environment.add_template("build.html", TEMPLATE)?;
    environment
        .get_template("build.html")?
        .render(context!(dashboard => dashboard))
}

fn watch(
    config_path: &Path,
    output_path: &Path,
    data_output_path: Option<&Path>,
) -> Result<(), Box<dyn Error>> {
    let watch_directory = config_path
        .parent()
        .expect("an absolute path always has a parent");
    let (sender, receiver) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = sender.send(event);
    })?;

    watcher.watch(watch_directory, RecursiveMode::NonRecursive)?;
    eprintln!("watching `{}` for changes", config_path.display());

    if let Err(error) = build_once(config_path, output_path, data_output_path) {
        eprintln!("error: {error}");
    }

    loop {
        wait_for_change(&receiver, config_path)?;

        if let Err(error) = build_once(config_path, output_path, data_output_path) {
            eprintln!("error: {error}");
        }
    }
}

fn wait_for_change(
    receiver: &Receiver<notify::Result<Event>>,
    config_path: &Path,
) -> Result<(), Box<dyn Error>> {
    loop {
        match receiver.recv()? {
            Ok(event) if event_affects_config(&event, config_path) => break,
            Ok(_) => {}
            Err(error) => eprintln!("warning: file watcher error: {error}"),
        }
    }

    let mut last_change = Instant::now();
    loop {
        let remaining = DEBOUNCE.saturating_sub(last_change.elapsed());

        match receiver.recv_timeout(remaining) {
            Ok(Ok(event)) if event_affects_config(&event, config_path) => {
                last_change = Instant::now();
            }
            Ok(Ok(_)) => {}
            Ok(Err(error)) => eprintln!("warning: file watcher error: {error}"),
            Err(RecvTimeoutError::Timeout) => return Ok(()),
            Err(RecvTimeoutError::Disconnected) => {
                return Err(io::Error::other("file watcher stopped unexpectedly").into());
            }
        }
    }
}

fn event_affects_config(event: &Event, config_path: &Path) -> bool {
    event.paths.iter().any(|path| path.as_path() == config_path)
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

#[cfg(test)]
mod tests;
