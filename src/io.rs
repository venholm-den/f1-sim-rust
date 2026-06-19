use crate::{model::DriverInput, simulate::DriverSummary};
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
