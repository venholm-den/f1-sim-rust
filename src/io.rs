use crate::{
    data_sources::{TeamPowerUnit, TrackProfile},
    model::DriverInput,
    simulate::DriverSummary,
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

pub fn write_summary(path: impl AsRef<Path>, summary: &[DriverSummary]) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create summary CSV {}", path.display()))?;
    for row in summary {
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
