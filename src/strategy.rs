use crate::{data_sources::RaceContext, model::DriverInput};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StrategyCandidate {
    pub driver: String,
    pub team: String,
    pub plan: String,
    pub stops: u8,
    pub score: f64,
    pub risk: f64,
}

pub fn predict_strategies(
    drivers: &[DriverInput],
    context: &RaceContext,
) -> Vec<StrategyCandidate> {
    let overtaking = context.overtaking_difficulty(0.55);
    let chaos = context.chaos_multiplier();
    let mut candidates = Vec::new();

    for driver in drivers {
        let one_stop_score = driver.strategy_score * 100.0 - overtaking * 8.0;
        let two_stop_score = driver.strategy_score * 96.0 + overtaking * 4.0 - chaos * 2.0;
        let soft_attack_score =
            driver.strategy_score * 92.0 + (1.0 - overtaking) * 6.0 - chaos * 3.0;

        candidates.push(candidate(
            driver,
            "Medium-Hard",
            1,
            one_stop_score,
            overtaking * 0.35,
        ));
        candidates.push(candidate(
            driver,
            "Medium-Hard-Soft",
            2,
            two_stop_score,
            chaos * 0.25,
        ));
        candidates.push(candidate(
            driver,
            "Soft-Medium-Hard",
            2,
            soft_attack_score,
            chaos * 0.35 + overtaking * 0.15,
        ));
    }

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
}

fn candidate(
    driver: &DriverInput,
    plan: &str,
    stops: u8,
    score: f64,
    risk: f64,
) -> StrategyCandidate {
    StrategyCandidate {
        driver: driver.driver.clone(),
        team: driver.team.clone(),
        plan: plan.to_string(),
        stops,
        score,
        risk: risk.clamp(0.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_sources::RaceContext;

    #[test]
    fn creates_three_candidates_per_driver() {
        let drivers = vec![DriverInput {
            driver: "AAA".to_string(),
            team: "Team".to_string(),
            grid: 1,
            pace_score: 0.9,
            strategy_score: 0.8,
            dnf_probability: 0.05,
            fantasy_price: None,
        }];

        let candidates = predict_strategies(&drivers, &RaceContext::default());

        assert_eq!(candidates.len(), 3);
    }
}
