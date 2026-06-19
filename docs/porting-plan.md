# Rust Porting Plan

The Rust project should be brought up in thin, testable slices. The Python project remains the reference implementation until parity is proven.

## Milestone 1: Simulation Core

- Driver input schema.
- Run configuration schema.
- Monte Carlo race simulation.
- Finish probability, podium probability, average finish, and DNF rate.
- Basic fantasy projection.
- CSV summary output.

## Milestone 2: Input Parity

- Track profile CSV.
- Fantasy price CSV.
- Team power-unit CSV.
- FIA document index compatibility.
- Output folder conventions matching the Python project.

## Milestone 3: Model Features

- Current-session feature model.
- Baseline race feature model.
- Reliability model.
- Weather modifiers.
- Race-control modifiers.
- Grid logic.

## Milestone 4: Strategy and Calibration

- Tyre inventory estimation.
- Candidate strategy scoring.
- Historical same-event strategy adjustment.
- Historical finish/DNF calibration artifacts.

## Milestone 5: App and Packaging

- Decide between Tauri and local web server UI.
- Recreate race setup, dashboard, model signals, track map, weather, strategy, data sources, and race review views.
- Add Windows build workflow.

