# F1 Sim Rust

Rust runtime foundation for the F1 race simulation and fantasy projection toolkit.

This repo is not trying to replace the full Python `f1-sim` project. The architecture is now hybrid: Python owns data collection, model training, reporting, Power BI prep, and bots; Rust owns the fast simulation engine, portable executable/runtime layer, and future Tauri desktop shell.

## Current Scope

- CLI entry point.
- Library crate plus CLI subcommands.
- GitHub Actions CI for format and tests.
- JSON run, output, and data-source configuration.
- CSV driver input.
- Feature-source CSV builder for generated driver inputs.
- OpenF1-to-driver-input builder from raw drivers, laps, and weather data.
- Track profile CSV input for overtaking, safety-car, and red-flag context.
- Team power-unit CSV input for reliability adjustment.
- OpenF1 REST ingestion for sessions, drivers, laps, and weather snapshots.
- Monte Carlo finish simulation.
- DNF probability handling with basic power-unit and track chaos modifiers.
- Tyre strategy candidate scoring.
- Basic fantasy points projection.
- CSV simulation summary output.
- Prediction snapshots under `outputs/history/`.
- Tiny local HTML dashboard for summary CSVs.
- Tag-triggered Windows release binary artifact workflow.

## Architecture Direction

- Python writes model-ready inputs.
- Rust runs fast simulation and writes typed outputs/snapshots.
- Python can still produce rich reports, Power BI prep, and Discord posts from Rust outputs.
- React/TypeScript should own the future UI.
- Tauri/Rust should own the portable desktop wrapper.

See [Hybrid Project Plan](docs/porting-plan.md) for the ownership split.

## Setup

Install Rust with `rustup`:

```powershell
winget install Rustlang.Rustup
```

Restart the terminal, then check:

```powershell
rustc --version
cargo --version
```

## Run

```powershell
cargo run -- simulate --config config/default_run_config.json --drivers data/sample_driver_inputs.csv
```

By default, the summary is written to `outputs/simulation_summary.csv`.

Override the simulation count:

```powershell
cargo run -- simulate --n-sims 50000
```

Write to a custom path:

```powershell
cargo run -- simulate --output outputs/custom_summary.csv
```

Build driver inputs from a feature-source CSV:

```powershell
cargo run -- build-inputs --source data/feature_source.csv --output data/generated_driver_inputs.csv
```

Predict tyre strategy candidates:

```powershell
cargo run -- strategy --drivers data/generated_driver_inputs.csv --output outputs/strategy_candidates.csv
```

Fetch OpenF1 session metadata:

```powershell
cargo run -- fetch-openf1 --year 2024 --output-dir outputs/openf1
```

Fetch OpenF1 session details by event/session:

```powershell
cargo run -- fetch-openf1 --year 2024 --event Monaco --session Q --output-dir outputs/openf1-monaco
```

Fetch OpenF1 session details when you already know the `session_key`:

```powershell
cargo run -- fetch-openf1 --year 2024 --session-key 9574 --output-dir outputs/openf1
```

Build driver inputs from fetched OpenF1 raw data:

```powershell
cargo run -- build-open-f1-inputs --session-key 9519 --input-dir outputs/openf1-monaco --output data/openf1_driver_inputs.csv
```

Or fetch and build in one command:

```powershell
cargo run -- build-open-f1-inputs --fetch --year 2024 --event Monaco --session Q --input-dir outputs/openf1-monaco --output data/openf1_driver_inputs.csv
```

Serve the generated summary dashboard:

```powershell
cargo run -- serve --summary outputs/openf1_summary.csv --strategy outputs/openf1_strategy.csv --sessions outputs/openf1-monaco/sessions.json --bind 127.0.0.1:7878
```

## Input Files

- `data/sample_driver_inputs.csv`: driver/team/grid/model scores and fantasy price inputs.
- `data/feature_source.csv`: intermediate feature inputs used by `build-inputs`.
- `data/track_profiles.csv`: overtaking difficulty, safety-car chance, and red-flag baseline by event.
- `data/team_power_units.csv`: team-to-power-unit mapping by season.

OpenF1 historical endpoints are queried from `https://api.openf1.org/v1` and do not require authentication for basic historical access.

## Snapshots

When `outputs.save_prediction_snapshot` is enabled in `config/default_run_config.json`, `simulate` also writes:

- `outputs/history/latest_prediction_snapshot.csv`
- `outputs/history/latest_prediction_snapshot.config.json`
- timestamped snapshot/config pairs for each run

## Release Builds

Pushing a tag that starts with `v` runs the `Release Build` workflow and uploads a Windows release executable artifact:

```powershell
git tag v0.1.0-alpha.1
git push origin v0.1.0-alpha.1
```

## Test

```powershell
cargo test
```

## Relationship to the Python Project

The Python repo remains the source of truth for F1 data collection, feature engineering, model training, historical calibration, reporting, Power BI prep, and Discord workflows. This Rust repo should provide a fast, typed runtime engine and portable app layer that consume Python-produced inputs.
