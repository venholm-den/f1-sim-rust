use crate::{openf1::OpenF1Session, simulate::DriverSummary, strategy::StrategyCandidate};
use anyhow::{Context, Result};
use std::{fs, path::PathBuf};
use tiny_http::{Header, Response, Server};

#[derive(Debug, Clone)]
pub struct DashboardPaths {
    pub summary: PathBuf,
    pub strategy: PathBuf,
    pub sessions: PathBuf,
}

pub fn serve_dashboard(paths: DashboardPaths, bind: &str) -> Result<()> {
    let server =
        Server::http(bind).map_err(|err| anyhow::anyhow!("failed to bind {bind}: {err}"))?;
    println!("Serving dashboard at http://{bind}");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let result = match url.as_str() {
            "/summary.csv" => serve_file(&paths.summary, "text/csv"),
            "/strategy.csv" => serve_file(&paths.strategy, "text/csv"),
            "/sessions.json" => serve_file(&paths.sessions, "application/json"),
            _ => serve_html(&paths),
        };

        let response = match result {
            Ok(response) => response,
            Err(err) => Response::from_string(format!("error: {err}")).with_status_code(500),
        };
        request.respond(response)?;
    }

    Ok(())
}

fn serve_file(path: &PathBuf, content_type: &str) -> Result<Response<std::io::Cursor<Vec<u8>>>> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(Response::from_data(bytes).with_header(text_header(content_type)?))
}

fn serve_html(paths: &DashboardPaths) -> Result<Response<std::io::Cursor<Vec<u8>>>> {
    let summary = read_csv::<DriverSummary>(&paths.summary).unwrap_or_default();
    let strategies = read_csv::<StrategyCandidate>(&paths.strategy).unwrap_or_default();
    let sessions = read_json::<Vec<OpenF1Session>>(&paths.sessions).unwrap_or_default();
    let mut body = String::from(
        r##"<!doctype html><html><head><meta charset="utf-8"><title>F1 Sim Rust</title>
<style>
body{font-family:Segoe UI,Arial,sans-serif;margin:32px;background:#101418;color:#eef2f5}
nav{display:flex;gap:12px;margin-bottom:20px}a{color:#7cc7ff}section{margin:28px 0}
table{border-collapse:collapse;width:100%;background:#151b22}th,td{border-bottom:1px solid #2a3340;padding:8px;text-align:left}
th{color:#9fb3c8}.num{text-align:right}.pill{display:inline-block;padding:2px 8px;border:1px solid #3a4655;border-radius:999px;color:#b7c6d6}
</style></head><body><h1>F1 Sim Rust Dashboard</h1>
<nav><a href="#summary">Summary</a><a href="#strategy">Strategy</a><a href="#sessions">OpenF1 Sessions</a><a href="/summary.csv">summary.csv</a><a href="/strategy.csv">strategy.csv</a><a href="/sessions.json">sessions.json</a></nav>"##,
    );

    body.push_str(r#"<section id="summary"><h2>Simulation Summary</h2><table><thead><tr><th>Driver</th><th>Team</th><th>PU</th><th class="num">Win</th><th class="num">Podium</th><th class="num">DNF</th><th class="num">Avg Finish</th><th class="num">Fantasy</th></tr></thead><tbody>"#);
    for row in summary {
        body.push_str(&format!(
            r#"<tr><td>{}</td><td>{}</td><td>{}</td><td class="num">{:.1}%</td><td class="num">{:.1}%</td><td class="num">{:.1}%</td><td class="num">{:.2}</td><td class="num">{:.2}</td></tr>"#,
            escape_html(&row.driver),
            escape_html(&row.team),
            escape_html(row.power_unit_supplier.as_deref().unwrap_or("-")),
            row.win_probability * 100.0,
            row.podium_probability * 100.0,
            row.dnf_probability * 100.0,
            row.average_finish,
            row.expected_fantasy_points
        ));
    }
    body.push_str("</tbody></table></section>");

    body.push_str(r#"<section id="strategy"><h2>Strategy Candidates</h2><table><thead><tr><th>Driver</th><th>Team</th><th>Plan</th><th class="num">Stops</th><th class="num">Score</th><th class="num">Risk</th></tr></thead><tbody>"#);
    for row in strategies.into_iter().take(60) {
        body.push_str(&format!(
            r#"<tr><td>{}</td><td>{}</td><td><span class="pill">{}</span></td><td class="num">{}</td><td class="num">{:.2}</td><td class="num">{:.1}%</td></tr>"#,
            escape_html(&row.driver),
            escape_html(&row.team),
            escape_html(&row.plan),
            row.stops,
            row.score,
            row.risk * 100.0
        ));
    }
    body.push_str("</tbody></table></section>");

    body.push_str(r#"<section id="sessions"><h2>OpenF1 Sessions</h2><table><thead><tr><th>Year</th><th>Event</th><th>Session</th><th>Key</th><th>Start</th></tr></thead><tbody>"#);
    for row in sessions.into_iter().take(80) {
        body.push_str(&format!(
            r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
            row.year,
            escape_html(
                row.meeting_name
                    .as_deref()
                    .or(row.country_name.as_deref())
                    .or(row.circuit_short_name.as_deref())
                    .unwrap_or("-")
            ),
            escape_html(&row.session_name),
            row.session_key,
            escape_html(row.date_start.as_deref().unwrap_or("-"))
        ));
    }
    body.push_str("</tbody></table></section></body></html>");

    Ok(Response::from_string(body).with_header(text_header("text/html; charset=utf-8")?))
}

fn read_csv<T>(path: &PathBuf) -> Result<Vec<T>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut rows = Vec::new();
    for row in reader.deserialize() {
        rows.push(row?);
    }
    Ok(rows)
}

fn read_json<T>(path: &PathBuf) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn text_header(value: &str) -> Result<Header> {
    Header::from_bytes("Content-Type", value)
        .map_err(|_| anyhow::anyhow!("failed to build content-type header"))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
