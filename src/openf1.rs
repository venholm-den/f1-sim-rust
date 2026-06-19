use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.openf1.org/v1";

#[derive(Debug, Clone)]
pub struct OpenF1Client {
    client: Client,
    base_url: String,
}

impl Default for OpenF1Client {
    fn default() -> Self {
        Self::new(BASE_URL)
    }
}

impl OpenF1Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    pub fn sessions(&self, year: u16) -> Result<Vec<OpenF1Session>> {
        self.get_json("sessions", &[("year", year.to_string())])
    }

    pub fn drivers(&self, session_key: i64) -> Result<Vec<OpenF1Driver>> {
        self.get_json("drivers", &[("session_key", session_key.to_string())])
    }

    pub fn laps(&self, session_key: i64) -> Result<Vec<OpenF1Lap>> {
        self.get_json("laps", &[("session_key", session_key.to_string())])
    }

    pub fn weather(&self, session_key: i64) -> Result<Vec<OpenF1Weather>> {
        self.get_json("weather", &[("session_key", session_key.to_string())])
    }

    fn get_json<T>(&self, endpoint: &str, params: &[(&str, String)]) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint);
        let response = self
            .client
            .get(&url)
            .query(params)
            .send()
            .with_context(|| format!("failed to request OpenF1 endpoint {endpoint}"))?
            .error_for_status()
            .with_context(|| format!("OpenF1 endpoint {endpoint} returned an error"))?;

        response
            .json()
            .with_context(|| format!("failed to parse OpenF1 endpoint {endpoint}"))
    }
}

pub fn select_session(
    sessions: &[OpenF1Session],
    event: &str,
    session: &str,
) -> Option<OpenF1Session> {
    let event_key = normalize(event);
    let session_key = normalize(session);

    sessions
        .iter()
        .filter(|candidate| {
            event_key.is_empty()
                || text_matches(candidate.meeting_name.as_deref(), &event_key)
                || text_matches(candidate.country_name.as_deref(), &event_key)
                || text_matches(candidate.circuit_short_name.as_deref(), &event_key)
                || text_matches(candidate.location.as_deref(), &event_key)
        })
        .filter(|candidate| {
            session_key.is_empty()
                || normalize(&candidate.session_name) == session_key
                || normalize(&candidate.session_type) == session_key
                || session_alias(&session_key)
                    .map(|alias| normalize(&candidate.session_name).contains(alias))
                    .unwrap_or(false)
        })
        .min_by_key(|candidate| candidate.date_start.clone())
        .cloned()
}

fn text_matches(value: Option<&str>, needle: &str) -> bool {
    value
        .map(|value| normalize(value).contains(needle))
        .unwrap_or(false)
}

fn session_alias(session: &str) -> Option<&'static str> {
    match session {
        "q" => Some("qualifying"),
        "r" => Some("race"),
        "fp1" => Some("practice 1"),
        "fp2" => Some("practice 2"),
        "fp3" => Some("practice 3"),
        "s" => Some("sprint"),
        "sq" => Some("sprint qualifying"),
        _ => None,
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenF1Session {
    pub session_key: i64,
    pub meeting_key: i64,
    pub year: u16,
    pub session_name: String,
    pub session_type: String,
    pub meeting_name: Option<String>,
    pub country_name: Option<String>,
    pub circuit_short_name: Option<String>,
    pub location: Option<String>,
    pub date_start: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenF1Driver {
    pub session_key: i64,
    pub driver_number: u16,
    pub broadcast_name: Option<String>,
    pub full_name: Option<String>,
    pub name_acronym: Option<String>,
    pub team_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenF1Lap {
    pub session_key: i64,
    pub driver_number: u16,
    pub lap_number: u16,
    pub lap_duration: Option<f64>,
    pub duration_sector_1: Option<f64>,
    pub duration_sector_2: Option<f64>,
    pub duration_sector_3: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenF1Weather {
    pub session_key: i64,
    pub air_temperature: Option<f64>,
    pub track_temperature: Option<f64>,
    pub rainfall: Option<f64>,
    pub humidity: Option<f64>,
    pub wind_speed: Option<f64>,
}
