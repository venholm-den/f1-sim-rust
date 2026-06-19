use serde::{Deserialize, Serialize};

pub const MODEL_INPUT_SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelInputFile {
    pub schema_version: String,
    pub source: Option<String>,
    pub run: ModelInputRun,
    pub drivers: Vec<DriverInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelInputRun {
    pub year: u16,
    pub event: String,
    pub session: String,
}

impl ModelInputFile {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.schema_version == MODEL_INPUT_SCHEMA_VERSION,
            "unsupported model input schema_version {}",
            self.schema_version
        );
        anyhow::ensure!(self.drivers.len() >= 2, "at least two drivers are required");
        for driver in &self.drivers {
            driver.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DriverInput {
    pub driver: String,
    pub team: String,
    pub grid: u8,
    pub pace_score: f64,
    pub strategy_score: f64,
    pub dnf_probability: f64,
    pub fantasy_price: Option<f64>,
}

impl DriverInput {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.driver.trim().is_empty(), "driver code is required");
        anyhow::ensure!(!self.team.trim().is_empty(), "team is required");
        anyhow::ensure!(self.grid >= 1, "grid must be at least 1");
        anyhow::ensure!(
            (0.0..=1.0).contains(&self.dnf_probability),
            "dnf_probability must be between 0 and 1"
        );
        Ok(())
    }
}
