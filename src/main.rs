mod config;
mod fantasy;
mod io;
mod model;
mod simulate;

use anyhow::Result;
use clap::Parser;
use config::AppConfig;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about = "Rust F1 race simulation prototype")]
struct Args {
    #[arg(long, default_value = "config/default_run_config.json")]
    config: PathBuf,

    #[arg(long, default_value = "data/sample_driver_inputs.csv")]
    drivers: PathBuf,

    #[arg(long, default_value = "outputs/simulation_summary.csv")]
    output: PathBuf,

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
    let summary = simulate::run_monte_carlo(&drivers, &config)?;
    io::write_summary(&args.output, &summary)?;

    println!(
        "Wrote {} driver summaries for {} {} {} to {}",
        summary.len(),
        config.run.year,
        config.run.event,
        config.run.session,
        args.output.display()
    );

    Ok(())
}
