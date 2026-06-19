use crate::simulate::DriverSummary;
use anyhow::{Context, Result};
use std::{fs, path::Path};
use tiny_http::{Header, Response, Server};

pub fn serve_summary(summary_path: impl AsRef<Path>, bind: &str) -> Result<()> {
    let summary_path = summary_path.as_ref().to_path_buf();
    let server =
        Server::http(bind).map_err(|err| anyhow::anyhow!("failed to bind {bind}: {err}"))?;
    println!("Serving dashboard at http://{bind}");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let result = if url == "/summary.csv" {
            serve_csv(&summary_path)
        } else {
            serve_html(&summary_path)
        };

        let response = match result {
            Ok(response) => response,
            Err(err) => Response::from_string(format!("error: {err}")).with_status_code(500),
        };
        request.respond(response)?;
    }

    Ok(())
}

fn serve_csv(summary_path: &Path) -> Result<Response<std::io::Cursor<Vec<u8>>>> {
    let bytes = fs::read(summary_path)
        .with_context(|| format!("failed to read {}", summary_path.display()))?;
    Ok(Response::from_data(bytes).with_header(text_header("text/csv")?))
}

fn serve_html(summary_path: &Path) -> Result<Response<std::io::Cursor<Vec<u8>>>> {
    let rows = read_summary(summary_path)?;
    let mut body = String::from(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>F1 Sim Rust</title>
<style>body{font-family:Segoe UI,Arial,sans-serif;margin:32px;background:#101418;color:#eef2f5}table{border-collapse:collapse;width:100%;background:#151b22}th,td{border-bottom:1px solid #2a3340;padding:8px;text-align:left}th{color:#9fb3c8}.num{text-align:right}a{color:#7cc7ff}</style>
</head><body><h1>F1 Sim Rust Dashboard</h1><p><a href="/summary.csv">Download CSV</a></p><table><thead><tr><th>Driver</th><th>Team</th><th>PU</th><th class="num">Win</th><th class="num">Podium</th><th class="num">DNF</th><th class="num">Avg Finish</th><th class="num">Fantasy</th></tr></thead><tbody>"#,
    );

    for row in rows {
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

    body.push_str("</tbody></table></body></html>");
    Ok(Response::from_string(body).with_header(text_header("text/html; charset=utf-8")?))
}

fn read_summary(path: &Path) -> Result<Vec<DriverSummary>> {
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("failed to read summary CSV {}", path.display()))?;
    let mut rows = Vec::new();
    for row in reader.deserialize() {
        rows.push(row?);
    }
    Ok(rows)
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
