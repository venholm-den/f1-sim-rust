use crate::{
    data_sources::{TeamPowerUnit, TrackProfile},
    features::FeatureSourceRow,
    model::{DriverInput, ModelInputFile},
    simulate::DriverSummary,
    strategy::StrategyCandidate,
};
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn read_driver_inputs(path: impl AsRef<Path>) -> Result<Vec<DriverInput>> {
    let path = path.as_ref();
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open driver input CSV {}", path.display()))?;
    let mut rows = Vec::new();

    for row in reader.deserialize() {
        let input: DriverInput =
            row.with_context(|| format!("failed to parse driver row in {}", path.display()))?;
        rows.push(input);
    }

    anyhow::ensure!(!rows.is_empty(), "driver input CSV contains no rows");
    Ok(rows)
}

pub fn read_track_profiles(path: impl AsRef<Path>) -> Result<Vec<TrackProfile>> {
    read_csv_rows(path, "track profiles")
}

pub fn read_team_power_units(path: impl AsRef<Path>) -> Result<Vec<TeamPowerUnit>> {
    read_csv_rows(path, "team power units")
}

pub fn read_feature_source(path: impl AsRef<Path>) -> Result<Vec<FeatureSourceRow>> {
    read_csv_rows(path, "feature source")
}

pub fn read_model_inputs(path: impl AsRef<Path>) -> Result<ModelInputFile> {
    let model_inputs: ModelInputFile = read_json(path)?;
    model_inputs.validate()?;
    Ok(model_inputs)
}

pub fn write_summary(path: impl AsRef<Path>, summary: &[DriverSummary]) -> Result<()> {
    write_csv_rows(path, summary, "summary")
}

pub fn write_driver_inputs(path: impl AsRef<Path>, drivers: &[DriverInput]) -> Result<()> {
    write_csv_rows(path, drivers, "driver inputs")
}

pub fn write_strategy_candidates(
    path: impl AsRef<Path>,
    candidates: &[StrategyCandidate],
) -> Result<()> {
    write_csv_rows(path, candidates, "strategy candidates")
}

pub fn write_json<T>(path: impl AsRef<Path>, value: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let raw = serde_json::to_string_pretty(value)?;
    fs::write(path, raw).with_context(|| format!("failed to write JSON {}", path.display()))?;
    Ok(())
}

pub fn read_json<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read JSON {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to parse JSON {}", path.display()))
}

fn write_csv_rows<T>(path: impl AsRef<Path>, rows: &[T], label: &str) -> Result<()>
where
    T: serde::Serialize,
{
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {label} CSV {}", path.display()))?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn read_csv_rows<T>(path: impl AsRef<Path>, label: &str) -> Result<Vec<T>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let path = path.as_ref();
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to open {label} CSV {}", path.display()))?;
    let mut rows = Vec::new();

    for row in reader.deserialize() {
        let input: T =
            row.with_context(|| format!("failed to parse {label} row in {}", path.display()))?;
        rows.push(input);
    }

    Ok(rows)
}
