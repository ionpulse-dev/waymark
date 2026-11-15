//! A small blocking HTTP client for the calls that the server makes: GitHub and client documents.
//! Call it from a blocking thread (`spawn_blocking`), never from an async task.

use serde_json::Value;
use std::time::Duration;

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(15)))
        .user_agent("waymark")
        .build()
        .into()
}

/// The status code and the JSON body (`Null` if the body is not JSON).
pub struct Reply {
    pub status: u16,
    pub json: Value,
}

fn read(mut r: ureq::http::Response<ureq::Body>) -> Result<Reply, String> {
    let status = r.status().as_u16();
    let text = r
        .body_mut()
        .with_config()
        .limit(1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())?;
    Ok(Reply {
        status,
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
    })
}

pub fn get_json(url: &str, bearer: Option<&str>) -> Result<Reply, String> {
    let mut req = agent().get(url).header("Accept", "application/json");
    if let Some(t) = bearer {
        req = req.header("Authorization", &format!("Bearer {t}"));
    }
    read(req.call().map_err(|e| e.to_string())?)
}

/// POST with no body and a bearer token (used for the GitHub App token).
pub fn post_empty(url: &str, bearer: &str) -> Result<Reply, String> {
    let req = agent()
        .post(url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", &format!("Bearer {bearer}"));
    read(req.send_empty().map_err(|e| e.to_string())?)
}

/// POST a form and read JSON (used for the GitHub sign-in code exchange).
pub fn post_form(url: &str, form: &[(&str, &str)]) -> Result<Reply, String> {
    let req = agent().post(url).header("Accept", "application/json");
    read(
        req.send_form(form.iter().copied())
            .map_err(|e| e.to_string())?,
    )
}
