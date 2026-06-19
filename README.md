# F1 Sim Rust

Rust port foundation for the F1 race simulation and fantasy projection toolkit.

This repo starts with the core shape of the Python `f1-sim` project: configuration, driver inputs, Monte Carlo race simulation, DNF risk, tyre/strategy pressure, fantasy scoring, and CSV outputs. The first milestone is intentionally small and buildable so the model can be ported in controlled layers.

## Current Scope

- CLI entry point.
- JSON run configuration.
- CSV driver input.
- Monte Carlo finish simulation.
- DNF probability handling.
- Basic fantasy points projection.
- CSV simulation summary output.

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
cargo run -- --config config/default_run_config.json --drivers data/sample_driver_inputs.csv --output outputs/simulation_summary.csv
```

Override the simulation count:

```powershell
cargo run -- --n-sims 50000
```

## Test

```powershell
cargo test
```

## Relationship to the Python Project

The Python repo remains the source of truth while this Rust port is brought up. The Rust version should stay honest about parity: do not claim a model feature is ported until the behavior, inputs, outputs, and tests exist here.

