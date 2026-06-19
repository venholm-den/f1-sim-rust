use crate::model::DriverInput;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureSourceRow {
    pub driver: String,
    pub team: String,
    pub grid: u8,
    pub quali_score: Option<f64>,
    pub race_score: Option<f64>,
    pub strategy_score: Option<f64>,
    pub reliability_score: Option<f64>,
    pub fantasy_price: Option<f64>,
}

pub fn build_driver_inputs(rows: &[FeatureSourceRow]) -> Vec<DriverInput> {
    rows.iter().map(build_driver_input).collect()
}

fn build_driver_input(row: &FeatureSourceRow) -> DriverInput {
    let quali = row.quali_score.unwrap_or(0.75);
    let race = row.race_score.unwrap_or(quali);
    let strategy = row.strategy_score.unwrap_or(0.75);
    let reliability = row.reliability_score.unwrap_or(0.95).clamp(0.0, 1.0);

    DriverInput {
        driver: row.driver.clone(),
        team: row.team.clone(),
        grid: row.grid,
        pace_score: (quali * 0.45 + race * 0.55).clamp(0.0, 1.0),
        strategy_score: strategy.clamp(0.0, 1.0),
        dnf_probability: ((1.0 - reliability) * 1.25).clamp(0.0, 0.35),
        fantasy_price: row.fantasy_price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_driver_input_from_feature_row() {
        let rows = vec![FeatureSourceRow {
            driver: "AAA".to_string(),
            team: "Team".to_string(),
            grid: 2,
            quali_score: Some(0.9),
            race_score: Some(0.8),
            strategy_score: Some(0.7),
            reliability_score: Some(0.96),
            fantasy_price: Some(12.0),
        }];

        let inputs = build_driver_inputs(&rows);

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].driver, "AAA");
        assert!(inputs[0].pace_score > 0.8);
        assert!(inputs[0].dnf_probability > 0.0);
    }
}
