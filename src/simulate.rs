use crate::{config::AppConfig, data_sources::RaceContext, fantasy, model::DriverInput};
use anyhow::Result;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DriverRaceResult {
    pub driver: String,
    pub grid_position: u8,
    pub finish_position: u8,
    pub dnf: bool,
}

#[derive(Debug, Default)]
struct DriverAccumulator {
    starts: u32,
    wins: u32,
    podiums: u32,
    dnfs: u32,
    finish_total: f64,
    fantasy_total: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DriverSummary {
    pub driver: String,
    pub team: String,
    pub power_unit_supplier: Option<String>,
    pub starts: u32,
    pub win_probability: f64,
    pub podium_probability: f64,
    pub dnf_probability: f64,
    pub average_finish: f64,
    pub expected_fantasy_points: f64,
    pub fantasy_price: Option<f64>,
    pub expected_points_per_price: Option<f64>,
}

pub fn run_monte_carlo(
    drivers: &[DriverInput],
    config: &AppConfig,
    context: &RaceContext,
) -> Result<Vec<DriverSummary>> {
    anyhow::ensure!(drivers.len() >= 2, "at least two drivers are required");
    for driver in drivers {
        driver.validate()?;
    }

    let mut rng = StdRng::seed_from_u64(config.run.random_seed);
    let normal = Normal::new(0.0, config.model.race_noise_seconds)?;
    let mut accumulators: HashMap<String, DriverAccumulator> = drivers
        .iter()
        .map(|driver| (driver.driver.clone(), DriverAccumulator::default()))
        .collect();

    for _ in 0..config.run.n_sims {
        let race = simulate_once(drivers, config, context, &normal, &mut rng);
        for result in race {
            let fantasy_points = fantasy::score_driver(&result, &config.fantasy);
            let acc = accumulators
                .get_mut(&result.driver)
                .expect("accumulator exists for every driver");
            acc.starts += 1;
            acc.wins += u32::from(result.finish_position == 1 && !result.dnf);
            acc.podiums += u32::from(result.finish_position <= 3 && !result.dnf);
            acc.dnfs += u32::from(result.dnf);
            acc.finish_total += result.finish_position as f64;
            acc.fantasy_total += fantasy_points;
        }
    }

    let mut summary: Vec<DriverSummary> = drivers
        .iter()
        .map(|driver| {
            let acc = accumulators
                .get(&driver.driver)
                .expect("accumulator exists for every driver");
            let starts = acc.starts.max(1);
            let expected_fantasy_points = acc.fantasy_total / starts as f64;
            DriverSummary {
                driver: driver.driver.clone(),
                team: driver.team.clone(),
                power_unit_supplier: context
                    .power_unit_supplier(&driver.team)
                    .map(ToString::to_string),
                starts: acc.starts,
                win_probability: acc.wins as f64 / starts as f64,
                podium_probability: acc.podiums as f64 / starts as f64,
                dnf_probability: acc.dnfs as f64 / starts as f64,
                average_finish: acc.finish_total / starts as f64,
                expected_fantasy_points,
                fantasy_price: driver.fantasy_price,
                expected_points_per_price: driver
                    .fantasy_price
                    .filter(|price| *price > 0.0)
                    .map(|price| expected_fantasy_points / price),
            }
        })
        .collect();

    summary.sort_by(|a, b| {
        a.average_finish
            .partial_cmp(&b.average_finish)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(summary)
}

fn simulate_once(
    drivers: &[DriverInput],
    config: &AppConfig,
    context: &RaceContext,
    normal: &Normal<f64>,
    rng: &mut StdRng,
) -> Vec<DriverRaceResult> {
    let overtaking_difficulty =
        context.overtaking_difficulty(config.run.default_overtaking_difficulty);
    let chaos_multiplier = context.chaos_multiplier();

    let mut scored: Vec<(usize, f64, bool)> = drivers
        .iter()
        .enumerate()
        .map(|(idx, driver)| {
            let pace_time = (1.0 - driver.pace_score).max(0.0) * 100.0;
            let strategy_time =
                (1.0 - driver.strategy_score).max(0.0) * config.model.strategy_loss_seconds;
            let grid_time =
                driver.grid as f64 * config.model.grid_loss_seconds * (1.0 + overtaking_difficulty);
            let noise = normal.sample(rng) * chaos_multiplier;
            let dnf_probability = (driver.dnf_probability
                * context.power_unit_dnf_multiplier(&driver.team)
                * chaos_multiplier.sqrt())
            .clamp(0.0, 0.95);
            let dnf = rng.gen_bool(dnf_probability);
            let dnf_penalty = if dnf {
                config.model.dnf_time_penalty_seconds
            } else {
                0.0
            };
            (
                idx,
                pace_time + strategy_time + grid_time + noise + dnf_penalty,
                dnf,
            )
        })
        .collect();

    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    scored
        .into_iter()
        .enumerate()
        .map(|(position_idx, (driver_idx, _race_time_seconds, dnf))| {
            let driver = &drivers[driver_idx];
            DriverRaceResult {
                driver: driver.driver.clone(),
                grid_position: driver.grid,
                finish_position: (position_idx + 1) as u8,
                dnf,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{DataConfig, FantasyConfig, ModelConfig, OutputConfig, RunConfig},
        data_sources::{RaceContext, TeamPowerUnit, TrackProfile},
    };
    use std::path::PathBuf;

    fn config() -> AppConfig {
        AppConfig {
            run: RunConfig {
                year: 2026,
                event: "test".to_string(),
                session: "Q".to_string(),
                n_sims: 100,
                random_seed: 7,
                default_overtaking_difficulty: 0.55,
            },
            outputs: OutputConfig {
                output_dir: PathBuf::from("outputs"),
            },
            data: DataConfig {
                track_profiles_path: PathBuf::from("data/track_profiles.csv"),
                team_power_units_path: PathBuf::from("data/team_power_units.csv"),
            },
            model: ModelConfig {
                race_noise_seconds: 1.0,
                grid_loss_seconds: 0.1,
                strategy_loss_seconds: 1.0,
                dnf_time_penalty_seconds: 900.0,
            },
            fantasy: FantasyConfig {
                dnf_penalty: -15.0,
                position_gain_points_per_place: 2.0,
                position_loss_points_per_place: -1.0,
            },
        }
    }

    fn context() -> RaceContext {
        RaceContext::new(
            "test",
            2026,
            vec![TrackProfile {
                event: "test".to_string(),
                overtaking_difficulty: 0.7,
                safety_car_chance: 0.3,
                red_flag_base_chance: 0.05,
            }],
            vec![TeamPowerUnit {
                year: 2026,
                team: "A".to_string(),
                power_unit_supplier: "Mercedes".to_string(),
            }],
        )
    }

    #[test]
    fn simulation_returns_one_summary_per_driver() {
        let drivers = vec![
            DriverInput {
                driver: "AAA".to_string(),
                team: "A".to_string(),
                grid: 1,
                pace_score: 0.95,
                strategy_score: 0.9,
                dnf_probability: 0.0,
                fantasy_price: Some(10.0),
            },
            DriverInput {
                driver: "BBB".to_string(),
                team: "B".to_string(),
                grid: 2,
                pace_score: 0.85,
                strategy_score: 0.8,
                dnf_probability: 0.0,
                fantasy_price: Some(8.0),
            },
        ];

        let summary = run_monte_carlo(&drivers, &config(), &context()).unwrap();

        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].starts, 100);
    }
}
