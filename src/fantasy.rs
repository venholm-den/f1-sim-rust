use crate::{config::FantasyConfig, simulate::DriverRaceResult};

const FINISH_POINTS: [f64; 10] = [25.0, 18.0, 15.0, 12.0, 10.0, 8.0, 6.0, 4.0, 2.0, 1.0];

pub fn score_driver(result: &DriverRaceResult, config: &FantasyConfig) -> f64 {
    if result.dnf {
        return config.dnf_penalty;
    }

    let finish_points = FINISH_POINTS
        .get(result.finish_position.saturating_sub(1) as usize)
        .copied()
        .unwrap_or(0.0);
    let position_delta = result.grid_position as i32 - result.finish_position as i32;
    let movement_points = if position_delta >= 0 {
        position_delta as f64 * config.position_gain_points_per_place
    } else {
        position_delta.unsigned_abs() as f64 * config.position_loss_points_per_place
    };

    finish_points + movement_points
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> FantasyConfig {
        FantasyConfig {
            fastest_lap_bonus: 5.0,
            dnf_penalty: -15.0,
            position_gain_points_per_place: 2.0,
            position_loss_points_per_place: -1.0,
        }
    }

    #[test]
    fn dnf_uses_penalty() {
        let result = DriverRaceResult {
            driver: "AAA".to_string(),
            team: "Team".to_string(),
            grid_position: 4,
            finish_position: 20,
            race_time_seconds: 900.0,
            dnf: true,
        };

        assert_eq!(score_driver(&result, &config()), -15.0);
    }
}

