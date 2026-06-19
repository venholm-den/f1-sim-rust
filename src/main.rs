use anyhow::Result;
use clap::{Parser, Subcommand};
use f1_sim_rust::{
    config::AppConfig, dashboard, data_sources::RaceContext, features, io, openf1,
    openf1::OpenF1Client, simulate, strategy,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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
    BuildOpenF1Inputs(BuildOpenF1InputsArgs),
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
struct BuildOpenF1InputsArgs {
    #[arg(long)]
    year: Option<u16>,

    #[arg(long)]
    event: Option<String>,

    #[arg(long)]
    session: Option<String>,

    #[arg(long)]
    session_key: Option<i64>,

    #[arg(long, default_value = "outputs/openf1")]
    input_dir: PathBuf,

    #[arg(long, default_value = "data/openf1_driver_inputs.csv")]
    output: PathBuf,

    #[arg(long)]
    fetch: bool,
}

#[derive(Debug, Parser)]
struct FetchOpenF1Args {
    #[arg(long)]
    year: u16,

    #[arg(long)]
    event: Option<String>,

    #[arg(long)]
    session: Option<String>,

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

    #[arg(long, default_value = "outputs/strategy_candidates.csv")]
    strategy: PathBuf,

    #[arg(long, default_value = "outputs/openf1/sessions.json")]
    sessions: PathBuf,

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
        Command::BuildOpenF1Inputs(args) => build_openf1_inputs_command(args),
        Command::FetchOpenF1(args) => fetch_openf1_command(args),
        Command::Strategy(args) => strategy_command(args),
        Command::Serve(args) => dashboard::serve_dashboard(
            dashboard::DashboardPaths {
                summary: args.summary,
                strategy: args.strategy,
                sessions: args.sessions,
            },
            &args.bind,
        ),
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
    if config.outputs.save_prediction_snapshot {
        write_prediction_snapshot(&config, &summary)?;
    }

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

fn build_openf1_inputs_command(args: BuildOpenF1InputsArgs) -> Result<()> {
    let session_key = if let Some(session_key) = args.session_key {
        session_key
    } else if args.fetch {
        let year = args
            .year
            .ok_or_else(|| anyhow::anyhow!("--year is required when --fetch is used"))?;
        let selected = fetch_selected_openf1_session(
            year,
            args.event.as_deref(),
            args.session.as_deref(),
            &args.input_dir,
        )?;
        selected.session_key
    } else {
        return Err(anyhow::anyhow!(
            "provide --session-key or use --fetch with --year"
        ));
    };

    let drivers = read_or_fetch_openf1_drivers(&args.input_dir, session_key, args.fetch)?;
    let laps = read_or_fetch_openf1_laps(&args.input_dir, session_key, args.fetch)?;
    let weather = read_or_fetch_openf1_weather(&args.input_dir, session_key, args.fetch)?;
    let generated = features::build_driver_inputs_from_openf1(&drivers, &laps, &weather);
    io::write_driver_inputs(&args.output, &generated)?;

    println!(
        "Wrote {} OpenF1-derived driver inputs to {}",
        generated.len(),
        args.output.display()
    );
    Ok(())
}

fn fetch_openf1_command(args: FetchOpenF1Args) -> Result<()> {
    if let Some(session_key) = args.session_key {
        fetch_openf1_session_data(session_key, &args.output_dir)?;
        println!("Fetched OpenF1 session data for session_key {session_key}");
    } else if args.event.is_some() || args.session.is_some() {
        let session = fetch_selected_openf1_session(
            args.year,
            args.event.as_deref(),
            args.session.as_deref(),
            &args.output_dir,
        )?;
        fetch_openf1_session_data(session.session_key, &args.output_dir)?;
        println!(
            "Fetched OpenF1 {} {} session_key {}",
            session
                .meeting_name
                .as_deref()
                .or(session.country_name.as_deref())
                .unwrap_or("event"),
            session.session_name,
            session.session_key
        );
    } else {
        let client = OpenF1Client::default();
        let sessions = client.sessions(args.year)?;
        io::write_json(args.output_dir.join("sessions.json"), &sessions)?;
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

fn write_prediction_snapshot(
    config: &AppConfig,
    summary: &[simulate::DriverSummary],
) -> Result<()> {
    let history_dir = config.outputs.output_dir.join("history");
    fs::create_dir_all(&history_dir)?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let snapshot_path = history_dir.join(format!("{timestamp}_prediction_snapshot.csv"));
    let config_path = history_dir.join(format!("{timestamp}_prediction_snapshot.config.json"));

    io::write_summary(&snapshot_path, summary)?;
    io::write_summary(history_dir.join("latest_prediction_snapshot.csv"), summary)?;
    io::write_json(&config_path, config)?;
    io::write_json(
        history_dir.join("latest_prediction_snapshot.config.json"),
        config,
    )?;
    Ok(())
}

fn fetch_selected_openf1_session(
    year: u16,
    event: Option<&str>,
    session: Option<&str>,
    output_dir: &Path,
) -> Result<openf1::OpenF1Session> {
    let client = OpenF1Client::default();
    let sessions = client.sessions(year)?;
    io::write_json(output_dir.join("sessions.json"), &sessions)?;

    match (event, session) {
        (Some(event), Some(session)) => openf1::select_session(&sessions, event, session)
            .ok_or_else(|| {
                anyhow::anyhow!("no OpenF1 session matched event={event:?} session={session:?}")
            }),
        _ => {
            println!("Fetched {} OpenF1 sessions for {}", sessions.len(), year);
            sessions
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("OpenF1 returned no sessions for {year}"))
        }
    }
}

fn fetch_openf1_session_data(session_key: i64, output_dir: &Path) -> Result<()> {
    let client = OpenF1Client::default();
    let drivers = client.drivers(session_key)?;
    let laps = client.laps(session_key)?;
    let weather = match client.weather(session_key) {
        Ok(weather) => weather,
        Err(err) => {
            eprintln!("warning: OpenF1 weather fetch failed for {session_key}: {err}");
            Vec::new()
        }
    };
    io::write_json(
        output_dir.join(format!("{session_key}_drivers.json")),
        &drivers,
    )?;
    io::write_json(output_dir.join(format!("{session_key}_laps.json")), &laps)?;
    io::write_json(
        output_dir.join(format!("{session_key}_weather.json")),
        &weather,
    )?;
    Ok(())
}

fn read_or_fetch_openf1_drivers(
    input_dir: &Path,
    session_key: i64,
    fetch: bool,
) -> Result<Vec<openf1::OpenF1Driver>> {
    let path = input_dir.join(format!("{session_key}_drivers.json"));
    if fetch || !path.exists() {
        fetch_openf1_session_data(session_key, input_dir)?;
    }
    io::read_json(path)
}

fn read_or_fetch_openf1_laps(
    input_dir: &Path,
    session_key: i64,
    fetch: bool,
) -> Result<Vec<openf1::OpenF1Lap>> {
    let path = input_dir.join(format!("{session_key}_laps.json"));
    if fetch || !path.exists() {
        fetch_openf1_session_data(session_key, input_dir)?;
    }
    io::read_json(path)
}

fn read_or_fetch_openf1_weather(
    input_dir: &Path,
    session_key: i64,
    fetch: bool,
) -> Result<Vec<openf1::OpenF1Weather>> {
    let path = input_dir.join(format!("{session_key}_weather.json"));
    if fetch || !path.exists() {
        fetch_openf1_session_data(session_key, input_dir)?;
    }
    io::read_json(path)
}
