use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub run: RunConfig,
    pub outputs: OutputConfig,
    pub data: DataConfig,
    pub model: ModelConfig,
    pub fantasy: FantasyConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunConfig {
    pub year: u16,
    pub event: String,
    pub session: String,
    pub n_sims: u32,
    pub random_seed: u64,
    pub default_overtaking_difficulty: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputConfig {
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataConfig {
    pub track_profiles_path: PathBuf,
    pub team_power_units_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    pub race_noise_seconds: f64,
    pub grid_loss_seconds: f64,
    pub strategy_loss_seconds: f64,
    pub dnf_time_penalty_seconds: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FantasyConfig {
    pub dnf_penalty: f64,
    pub position_gain_points_per_place: f64,
    pub position_loss_points_per_place: f64,
}

impl AppConfig {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
    }
}
