use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct TrackProfile {
    #[serde(rename = "Event")]
    pub event: String,
    #[serde(rename = "OvertakingDifficulty")]
    pub overtaking_difficulty: f64,
    #[serde(rename = "SafetyCarChance")]
    pub safety_car_chance: f64,
    #[serde(rename = "RedFlagBaseChance")]
    pub red_flag_base_chance: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamPowerUnit {
    #[serde(rename = "Year")]
    pub year: u16,
    #[serde(rename = "Team")]
    pub team: String,
    #[serde(rename = "PowerUnitSupplier")]
    pub power_unit_supplier: String,
}

#[derive(Debug, Clone, Default)]
pub struct RaceContext {
    pub track_profile: Option<TrackProfile>,
    power_units_by_team: HashMap<String, TeamPowerUnit>,
}

impl RaceContext {
    pub fn new(
        event: &str,
        year: u16,
        track_profiles: Vec<TrackProfile>,
        power_units: Vec<TeamPowerUnit>,
    ) -> Self {
        let track_profile = find_track_profile(event, track_profiles);
        let power_units_by_team = latest_power_units_by_team(year, power_units);

        Self {
            track_profile,
            power_units_by_team,
        }
    }

    pub fn overtaking_difficulty(&self, fallback: f64) -> f64 {
        self.track_profile
            .as_ref()
            .map(|profile| profile.overtaking_difficulty)
            .unwrap_or(fallback)
            .clamp(0.0, 1.0)
    }

    pub fn chaos_multiplier(&self) -> f64 {
        self.track_profile
            .as_ref()
            .map(|profile| {
                1.0 + profile.safety_car_chance.clamp(0.0, 1.0) * 0.35
                    + profile.red_flag_base_chance.clamp(0.0, 1.0) * 1.5
            })
            .unwrap_or(1.0)
    }

    pub fn power_unit_dnf_multiplier(&self, team: &str) -> f64 {
        self.power_units_by_team
            .get(&normalize_key(team))
            .map(|record| supplier_reliability_multiplier(&record.power_unit_supplier))
            .unwrap_or(1.0)
    }

    pub fn power_unit_supplier(&self, team: &str) -> Option<&str> {
        self.power_units_by_team
            .get(&normalize_key(team))
            .map(|record| record.power_unit_supplier.as_str())
    }
}

fn find_track_profile(event: &str, profiles: Vec<TrackProfile>) -> Option<TrackProfile> {
    let event_key = normalize_key(event);
    profiles
        .into_iter()
        .find(|profile| normalize_key(&profile.event) == event_key)
}

fn latest_power_units_by_team(
    year: u16,
    records: Vec<TeamPowerUnit>,
) -> HashMap<String, TeamPowerUnit> {
    let mut by_team = HashMap::new();

    for record in records.into_iter().filter(|record| record.year <= year) {
        let key = normalize_key(&record.team);
        let replace = by_team
            .get(&key)
            .map(|existing: &TeamPowerUnit| record.year > existing.year)
            .unwrap_or(true);
        if replace {
            by_team.insert(key, record);
        }
    }

    by_team
}

fn supplier_reliability_multiplier(supplier: &str) -> f64 {
    match normalize_key(supplier).as_str() {
        "mercedes" => 0.95,
        "ferrari" => 1.0,
        "honda rbpt" | "honda" | "rbpt" => 1.05,
        "renault" => 1.1,
        _ => 1.0,
    }
}

fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_latest_power_unit_at_or_before_year() {
        let context = RaceContext::new(
            "Monaco Grand Prix",
            2026,
            vec![],
            vec![
                TeamPowerUnit {
                    year: 2025,
                    team: "McLaren".to_string(),
                    power_unit_supplier: "Mercedes".to_string(),
                },
                TeamPowerUnit {
                    year: 2027,
                    team: "McLaren".to_string(),
                    power_unit_supplier: "Honda".to_string(),
                },
            ],
        );

        assert_eq!(context.power_unit_supplier("McLaren"), Some("Mercedes"));
    }
}
