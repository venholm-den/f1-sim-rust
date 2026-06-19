mod config;
mod data_sources;
mod fantasy;
mod io;
mod model;
mod simulate;

use anyhow::Result;
use clap::Parser;
use config::AppConfig;
use data_sources::RaceContext;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(version, about = "Rust F1 race simulation prototype")]
struct Args {
    #[arg(long, default_value = "config/default_run_config.json")]
    config: PathBuf,

    #[arg(long, default_value = "data/sample_driver_inputs.csv")]
    drivers: PathBuf,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long)]
    n_sims: Option<u32>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut config = AppConfig::from_path(&args.config)?;
    if let Some(n_sims) = args.n_sims {
        config.run.n_sims = n_sims;
    }

    let drivers = io::read_driver_inputs(&args.drivers)?;
    let track_profiles = io::read_track_profiles(&config.data.track_profiles_path)?;
    let power_units = io::read_team_power_units(&config.data.team_power_units_path)?;
    let context = RaceContext::new(
        &config.run.event,
        config.run.year,
        track_profiles,
        power_units,
    );
    let output = args
        .output
        .unwrap_or_else(|| default_summary_path(&config.outputs.output_dir));

    let summary = simulate::run_monte_carlo(&drivers, &config, &context)?;
    io::write_summary(&output, &summary)?;

    println!(
        "Wrote {} driver summaries for {} {} {} to {}",
        summary.len(),
        config.run.year,
        config.run.event,
        config.run.session,
        output.display()
    );

    Ok(())
}

fn default_summary_path(output_dir: &Path) -> PathBuf {
    output_dir.join("simulation_summary.csv")
}
