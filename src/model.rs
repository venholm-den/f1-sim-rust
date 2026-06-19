use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
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
