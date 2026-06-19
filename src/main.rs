use anyhow::Result;
use clap::{Parser, Subcommand};
use f1_sim_rust::{
    config::AppConfig, dashboard, data_sources::RaceContext, features, io, openf1::OpenF1Client,
    simulate, strategy,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(version, about = "Rust F1 race simulation prototype")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Simulate(SimulateArgs),
    BuildInputs(BuildInputsArgs),
    #[command(name = "fetch-openf1")]
    FetchOpenF1(FetchOpenF1Args),
    Strategy(StrategyArgs),
    Serve(ServeArgs),
}

#[derive(Debug, Parser)]
struct SimulateArgs {
    #[arg(long, default_value = "config/default_run_config.json")]
    config: PathBuf,

    #[arg(long, default_value = "data/sample_driver_inputs.csv")]
    drivers: PathBuf,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long)]
    n_sims: Option<u32>,
}

#[derive(Debug, Parser)]
struct BuildInputsArgs {
    #[arg(long, default_value = "data/feature_source.csv")]
    source: PathBuf,

    #[arg(long, default_value = "data/generated_driver_inputs.csv")]
    output: PathBuf,
}

#[derive(Debug, Parser)]
struct FetchOpenF1Args {
    #[arg(long)]
    year: u16,

    #[arg(long)]
    session_key: Option<i64>,

    #[arg(long, default_value = "outputs/openf1")]
    output_dir: PathBuf,
}

#[derive(Debug, Parser)]
struct StrategyArgs {
    #[arg(long, default_value = "config/default_run_config.json")]
    config: PathBuf,

    #[arg(long, default_value = "data/sample_driver_inputs.csv")]
    drivers: PathBuf,

    #[arg(long, default_value = "outputs/strategy_candidates.csv")]
    output: PathBuf,
}

#[derive(Debug, Parser)]
struct ServeArgs {
    #[arg(long, default_value = "outputs/simulation_summary.csv")]
    summary: PathBuf,

    #[arg(long, default_value = "127.0.0.1:7878")]
    bind: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Simulate(SimulateArgs {
        config: PathBuf::from("config/default_run_config.json"),
        drivers: PathBuf::from("data/sample_driver_inputs.csv"),
        output: None,
        n_sims: None,
    })) {
        Command::Simulate(args) => simulate_command(args),
        Command::BuildInputs(args) => build_inputs_command(args),
        Command::FetchOpenF1(args) => fetch_openf1_command(args),
        Command::Strategy(args) => strategy_command(args),
        Command::Serve(args) => dashboard::serve_summary(args.summary, &args.bind),
    }
}

fn simulate_command(args: SimulateArgs) -> Result<()> {
    let mut config = AppConfig::from_path(&args.config)?;
    if let Some(n_sims) = args.n_sims {
        config.run.n_sims = n_sims;
    }

    let drivers = io::read_driver_inputs(&args.drivers)?;
    let context = race_context(&config)?;
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

fn build_inputs_command(args: BuildInputsArgs) -> Result<()> {
    let rows = io::read_feature_source(&args.source)?;
    let drivers = features::build_driver_inputs(&rows);
    io::write_driver_inputs(&args.output, &drivers)?;
    println!(
        "Wrote {} generated driver inputs to {}",
        drivers.len(),
        args.output.display()
    );
    Ok(())
}

fn fetch_openf1_command(args: FetchOpenF1Args) -> Result<()> {
    let client = OpenF1Client::default();
    let sessions = client.sessions(args.year)?;
    io::write_json(args.output_dir.join("sessions.json"), &sessions)?;

    if let Some(session_key) = args.session_key {
        let drivers = client.drivers(session_key)?;
        let laps = client.laps(session_key)?;
        let weather = client.weather(session_key)?;
        io::write_json(
            args.output_dir.join(format!("{session_key}_drivers.json")),
            &drivers,
        )?;
        io::write_json(
            args.output_dir.join(format!("{session_key}_laps.json")),
            &laps,
        )?;
        io::write_json(
            args.output_dir.join(format!("{session_key}_weather.json")),
            &weather,
        )?;
        println!("Fetched OpenF1 session data for session_key {session_key}");
    } else {
        println!(
            "Fetched {} OpenF1 sessions for {}",
            sessions.len(),
            args.year
        );
    }

    Ok(())
}

fn strategy_command(args: StrategyArgs) -> Result<()> {
    let config = AppConfig::from_path(&args.config)?;
    let drivers = io::read_driver_inputs(&args.drivers)?;
    let context = race_context(&config)?;
    let candidates = strategy::predict_strategies(&drivers, &context);
    io::write_strategy_candidates(&args.output, &candidates)?;
    println!(
        "Wrote {} strategy candidates to {}",
        candidates.len(),
        args.output.display()
    );
    Ok(())
}

fn race_context(config: &AppConfig) -> Result<RaceContext> {
    let track_profiles = io::read_track_profiles(&config.data.track_profiles_path)?;
    let power_units = io::read_team_power_units(&config.data.team_power_units_path)?;
    Ok(RaceContext::new(
        &config.run.event,
        config.run.year,
        track_profiles,
        power_units,
    ))
}

fn default_summary_path(output_dir: &Path) -> PathBuf {
    output_dir.join("simulation_summary.csv")
}
