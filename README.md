# F1 Sim Rust

Rust port foundation for the F1 race simulation and fantasy projection toolkit.

This repo starts with the core shape of the Python `f1-sim` project: configuration, driver inputs, Monte Carlo race simulation, DNF risk, tyre/strategy pressure, fantasy scoring, and CSV outputs. The first milestone is intentionally small and buildable so the model can be ported in controlled layers.

## Current Scope

- CLI entry point.
- Library crate plus CLI subcommands.
- GitHub Actions CI for format and tests.
- JSON run, output, and data-source configuration.
- CSV driver input.
- Feature-source CSV builder for generated driver inputs.
- Track profile CSV input for overtaking, safety-car, and red-flag context.
- Team power-unit CSV input for reliability adjustment.
- OpenF1 REST ingestion for sessions, drivers, laps, and weather snapshots.
- Monte Carlo finish simulation.
- DNF probability handling with basic power-unit and track chaos modifiers.
- Tyre strategy candidate scoring.
- Basic fantasy points projection.
- CSV simulation summary output.
- Tiny local HTML dashboard for summary CSVs.

## Planned Porting Phases

1. Core simulation parity with the Python project.
2. Data-source layer for FastF1/OpenF1-compatible inputs.
3. Tyre strategy model and historical same-event adjustment.
4. Historical calibration artifacts and model-signal reporting.
5. Local app shell, likely using Tauri or a lightweight Rust web server plus static UI.
6. Packaging workflow for Windows portable builds.

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

Fetch OpenF1 session details when you know the `session_key`:

```powershell
cargo run -- fetch-openf1 --year 2024 --session-key 9574 --output-dir outputs/openf1
```

Serve the generated summary dashboard:

```powershell
cargo run -- serve --summary outputs/simulation_summary.csv --bind 127.0.0.1:7878
```

## Input Files

- `data/sample_driver_inputs.csv`: driver/team/grid/model scores and fantasy price inputs.
- `data/feature_source.csv`: intermediate feature inputs used by `build-inputs`.
- `data/track_profiles.csv`: overtaking difficulty, safety-car chance, and red-flag baseline by event.
- `data/team_power_units.csv`: team-to-power-unit mapping by season.

OpenF1 historical endpoints are queried from `https://api.openf1.org/v1` and do not require authentication for basic historical access.

## Test

```powershell
cargo test
```

## Relationship to the Python Project

The Python repo remains the source of truth while this Rust port is brought up. The Rust version should stay honest about parity: do not claim a model feature is ported until the behavior, inputs, outputs, and tests exist here.
