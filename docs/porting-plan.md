# Hybrid Project Plan

The goal is no longer to rewrite the whole Python project in Rust. The better architecture is a hybrid stack:

| Project area | Best choice |
| --- | --- |
| F1 data collection | Python |
| Model training | Python |
| Race simulation prototype | Python |
| Fast simulation engine | Rust |
| Portable `.exe` app shell | Rust/Tauri |
| Data reports | Python |
| Power BI prep | Python |
| Local network monitoring backend | Rust or Python |
| Discord bots | Python |
| Image/audio processing tools | Rust if performance matters |
| Web UI frontend | React/TypeScript |
| Desktop wrapper | Tauri/Rust |

## What Rust Owns

Rust should focus on fast, portable, user-facing runtime pieces:

- High-throughput Monte Carlo simulation.
- Stable typed config and input/output contracts.
- A portable desktop app shell through Tauri.
- Small local backend endpoints for the desktop app.
- Release packaging and Windows executable builds.
- Performance-sensitive utilities when Python becomes the bottleneck.

## What Python Owns

Python remains the source of truth for data science and reporting:

- FastF1/OpenF1 data collection and cache workflows.
- Feature engineering while the model is still changing.
- Model training, historical calibration, and backtesting.
- Report generation and rich data exports.
- Power BI prep and data-shaping outputs.
- Discord bot/report posting workflows.

## Interface Contract

The handoff between Python and Rust should be file/API based, not duplicated logic:

- Python writes driver/model input CSV or JSON.
- Rust reads typed inputs and runs fast simulation.
- Rust writes simulation summaries, snapshots, and strategy candidates.
- Python can post-process Rust outputs into reports, dashboards, Power BI files, or Discord posts.
- The Tauri app can call Rust commands directly and shell out to Python workflows where needed.

## Current Rust Status

- [x] CLI entry point.
- [x] Library crate plus CLI subcommands.
- [x] GitHub Actions CI for `cargo fmt --check` and `cargo test`.
- [x] JSON run, output, and data-source configuration.
- [x] CSV driver input schema.
- [x] Shared JSON model-input contract in `schemas/model_inputs.schema.json`.
- [x] Rust `simulate --model-inputs` primary runtime path.
- [x] Rust `simulate-batch` for many exported model-input files.
- [x] Feature-source CSV to generated driver-input pipeline.
- [x] OpenF1 raw data to generated driver-input pipeline.
- [x] Track profile CSV input.
- [x] Team power-unit CSV input.
- [x] OpenF1 REST ingestion for sessions, drivers, laps, and weather snapshots.
- [x] Event/session selection for OpenF1 fetches.
- [x] Monte Carlo finish simulation.
- [x] Basic fantasy points projection.
- [x] Initial tyre strategy candidate scoring.
- [x] Prediction snapshots for simulation outputs and run config.
- [x] Tiny local dashboard for summary, strategy, and OpenF1 sessions.
- [x] Tag-triggered Windows release executable artifact workflow.

## Next Rust Milestones

1. Add JSON schema validation to the Python exporter test path.
2. Wire the Python exporter into the normal post-run workflow.
3. Replace the temporary local dashboard with a Tauri shell and React frontend.
4. Expose Rust simulation commands to Tauri through a thin command API.
5. Add release packaging that bundles config, sample data, and the Tauri app.

## Deferred From Rust

These should not be rebuilt in Rust unless there is a clear performance or packaging reason:

- FastF1 cache/data extraction parity.
- Historical ML model training.
- Rich matplotlib-style report generation.
- Discord posting.
- Power BI shaping.
- Rapid model experimentation.
