use crate::{
    model::DriverInput,
    openf1::{OpenF1Driver, OpenF1Lap, OpenF1Weather},
};
use serde::Deserialize;
use std::collections::HashMap;

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

pub fn build_driver_inputs_from_openf1(
    drivers: &[OpenF1Driver],
    laps: &[OpenF1Lap],
    weather: &[OpenF1Weather],
) -> Vec<DriverInput> {
    let driver_lookup: HashMap<u16, &OpenF1Driver> = drivers
        .iter()
        .map(|driver| (driver.driver_number, driver))
        .collect();
    let mut lap_stats: HashMap<u16, DriverLapStats> = HashMap::new();

    for lap in laps
        .iter()
        .filter(|lap| lap.lap_duration.unwrap_or(0.0) > 0.0)
    {
        let stats = lap_stats.entry(lap.driver_number).or_default();
        let duration = lap.lap_duration.unwrap();
        stats.lap_count += 1;
        stats.best_lap = Some(
            stats
                .best_lap
                .map(|best| duration.min(best))
                .unwrap_or(duration),
        );
        stats.total_lap += duration;
    }

    let fastest_lap = lap_stats
        .values()
        .filter_map(|stats| stats.best_lap)
        .fold(f64::INFINITY, f64::min);
    let rainfall_seen = weather
        .iter()
        .any(|sample| sample.rainfall.unwrap_or(0.0) > 0.0);

    let mut rows: Vec<(u16, DriverInput)> = lap_stats
        .into_iter()
        .filter_map(|(driver_number, stats)| {
            let driver = driver_lookup.get(&driver_number)?;
            let best_lap = stats.best_lap?;
            let pace_score = if fastest_lap.is_finite() {
                (fastest_lap / best_lap).clamp(0.0, 1.0)
            } else {
                0.75
            };
            let average_lap = stats.total_lap / stats.lap_count.max(1) as f64;
            let consistency = (best_lap / average_lap).clamp(0.0, 1.0);
            let strategy_score = if rainfall_seen {
                consistency * 0.92
            } else {
                consistency
            };
            let reliability_score =
                (0.88 + (stats.lap_count as f64 / 40.0).min(0.1)).clamp(0.0, 1.0);

            Some((
                driver_number,
                DriverInput {
                    driver: driver_code(driver, driver_number),
                    team: driver
                        .team_name
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    grid: 1,
                    pace_score,
                    strategy_score,
                    dnf_probability: ((1.0 - reliability_score) * 1.25).clamp(0.0, 0.35),
                    fantasy_price: None,
                },
            ))
        })
        .collect();

    rows.sort_by(|a, b| {
        b.1.pace_score
            .partial_cmp(&a.1.pace_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    rows.into_iter()
        .enumerate()
        .map(|(idx, (_, mut driver))| {
            driver.grid = (idx + 1) as u8;
            driver
        })
        .collect()
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

#[derive(Debug, Default)]
struct DriverLapStats {
    lap_count: u32,
    best_lap: Option<f64>,
    total_lap: f64,
}

fn driver_code(driver: &OpenF1Driver, driver_number: u16) -> String {
    driver
        .name_acronym
        .clone()
        .or_else(|| driver.broadcast_name.clone())
        .or_else(|| driver.full_name.clone())
        .unwrap_or_else(|| driver_number.to_string())
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

    #[test]
    fn builds_driver_inputs_from_openf1_laps() {
        let drivers = vec![OpenF1Driver {
            session_key: 1,
            driver_number: 1,
            broadcast_name: None,
            full_name: Some("Example Driver".to_string()),
            name_acronym: Some("EXD".to_string()),
            team_name: Some("Example".to_string()),
        }];
        let laps = vec![OpenF1Lap {
            session_key: 1,
            driver_number: 1,
            lap_number: 1,
            lap_duration: Some(80.0),
            duration_sector_1: None,
            duration_sector_2: None,
            duration_sector_3: None,
        }];

        let inputs = build_driver_inputs_from_openf1(&drivers, &laps, &[]);

        assert_eq!(inputs[0].driver, "EXD");
        assert_eq!(inputs[0].grid, 1);
    }
}
