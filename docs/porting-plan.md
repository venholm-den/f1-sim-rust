# Rust Porting Plan

The Rust project should be brought up in thin, testable slices. The Python project remains the reference implementation until parity is proven.

## Milestone 1: Simulation Core

- [x] Driver input schema.
- [x] Run configuration schema.
- [x] Monte Carlo race simulation.
- [x] Finish probability, podium probability, average finish, and DNF rate.
- [x] Basic fantasy projection.
- [x] CSV summary output.

## Milestone 2: Input Parity

- [x] Track profile CSV.
- [x] Fantasy price field in driver input and generated feature input.
- [x] Team power-unit CSV.
- FIA document index compatibility.
- [x] Output folder conventions matching the Python project.
- [x] Feature-source CSV to generated driver-input pipeline.

## Milestone 3: Model Features

- Partial current-session feature model from feature-source CSV.
- Baseline race feature model.
- Partial reliability model from per-driver DNF probability and power-unit supplier.
- Weather modifiers.
- Race-control modifiers.
- Partial grid logic with overtaking difficulty.
- OpenF1 REST ingestion for sessions, drivers, laps, and weather snapshots.

## Milestone 4: Strategy and Calibration

- Tyre inventory estimation.
- [x] Initial candidate strategy scoring.
- Historical same-event strategy adjustment.
- Historical finish/DNF calibration artifacts.

## Milestone 5: App and Packaging

- Decide between Tauri and local web server UI.
- [x] Tiny local web server dashboard for simulation summaries.
- Recreate race setup, model signals, track map, weather, strategy, data sources, and race review views.
- Add Windows build workflow.

## Milestone 6: Repository Automation

- [x] GitHub Actions CI for `cargo fmt --check` and `cargo test`.
- Add release packaging workflow once the binary/app shape settles.
