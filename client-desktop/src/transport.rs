use std::time::Duration;

use anyhow::{Result, anyhow};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::event::ActivityEnvelope;

#[derive(Debug, Deserialize)]
struct AgentResponse {
    #[serde(default)]
    ok: Option<bool>,
    #[serde(default)]
    instruction: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    ack: Option<String>,
    #[serde(default)]
    server_time: Option<String>,
    #[serde(default)]
    refresh_config: Option<bool>,
}

pub async fn run_transport(
    server_api_base_url: String,
    agent_api_prefix: String,
    api_token: String,
    device_id: String,
    agent_name: String,
    mut rx: mpsc::UnboundedReceiver<ActivityEnvelope>,
) -> Result<()> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    let max_retry_delay = Duration::from_secs(30);
    let mut retry_delay = Duration::from_secs(2);
    let mut pending: Option<ActivityEnvelope> = None;

    loop {
        let event = match pending.take() {
            Some(event) => event,
            None => {
                let Some(event) = rx.recv().await else {
                    info!("event channel closed, transport exiting");
                    return Ok(());
                };
                event
            }
        };

        let endpoint = endpoint_for(&server_api_base_url, &agent_api_prefix, event.message_type)?;
        match send_event(&client, &endpoint, &api_token, &device_id, &agent_name, &event).await {
            Ok(()) => {
                retry_delay = Duration::from_secs(2);
            }
            Err(err) => {
                warn!(
                    error = %err,
                    endpoint = %endpoint,
                    delay_secs = retry_delay.as_secs(),
                    "event delivery failed, scheduling retry"
                );
                pending = Some(event);
                sleep(retry_delay).await;
                retry_delay = std::cmp::min(max_retry_delay, retry_delay.saturating_mul(2));
            }
        }
    }
}

fn endpoint_for(server_api_base_url: &str, agent_api_prefix: &str, message_type: &str) -> Result<String> {
    let path = match message_type {
        "activity" => "/activity",
        "status" => "/status",
        other => return Err(anyhow!("unsupported agent message type: {other}")),
    };

    let prefix = if agent_api_prefix.is_empty() {
        "/api/agent"
    } else {
        agent_api_prefix
    };

    Ok(format!("{server_api_base_url}{prefix}{path}"))
}

async fn send_event(
    client: &Client,
    endpoint: &str,
    api_token: &str,
    device_id: &str,
    agent_name: &str,
    event: &ActivityEnvelope,
) -> Result<()> {
    let response = client
        .post(endpoint)
        .bearer_auth(api_token)
        .header("x-eyes-on-me-device-id", device_id)
        .header("x-eyes-on-me-agent-name", agent_name)
        .header("x-eyes-on-me-message-type", event.message_type)
        .header("x-eyes-on-me-source", event.payload.source)
        .json(event)
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        let response_text = response.text().await.unwrap_or_default();
        handle_success_response(endpoint, status, &response_text, event);
        return Ok(());
    }

    let detail = response
        .text()
        .await
        .unwrap_or_else(|_| "failed to read error response".to_string());

    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return Err(anyhow!(
            "server rejected agent token: HTTP {status} {detail}"
        ));
    }

    Err(anyhow!("server returned HTTP {status}: {detail}"))
}

fn handle_success_response(
    endpoint: &str,
    status: StatusCode,
    response_text: &str,
    event: &ActivityEnvelope,
) {
    if response_text.trim().is_empty() {
        info!(
            endpoint = %endpoint,
            status = %status,
            app_id = %event.payload.app.id,
            browser_name = event.payload.browser.as_ref().map(|browser| browser.name.as_str()).unwrap_or("n/a"),
            pid = event.payload.app.pid,
            "event sent"
        );
        return;
    }

    match serde_json::from_str::<AgentResponse>(response_text) {
        Ok(payload) => {
            if let Some(instruction) = payload.instruction.as_deref().or(payload.command.as_deref()) {
                warn!(instruction = instruction, endpoint = %endpoint, "server returned unsupported instruction; ignored");
            }

            info!(
                endpoint = %endpoint,
                status = %status,
                ok = payload.ok.unwrap_or(true),
                ack = payload.ack.as_deref().unwrap_or(""),
                server_time = payload.server_time.as_deref().unwrap_or(""),
                refresh_config = payload.refresh_config.unwrap_or(false),
                app_id = %event.payload.app.id,
                browser_name = event.payload.browser.as_ref().map(|browser| browser.name.as_str()).unwrap_or("n/a"),
                pid = event.payload.app.pid,
                "event sent"
            );
        }
        Err(_) => {
            info!(
                endpoint = %endpoint,
                status = %status,
                app_id = %event.payload.app.id,
                browser_name = event.payload.browser.as_ref().map(|browser| browser.name.as_str()).unwrap_or("n/a"),
                pid = event.payload.app.pid,
                response_body = response_text,
                "event sent with non-json response body"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::endpoint_for;

    #[test]
    fn resolves_activity_endpoint() {
        let endpoint = endpoint_for("http://127.0.0.1:8787", "", "activity").unwrap();
        assert_eq!(endpoint, "http://127.0.0.1:8787/api/agent/activity");
    }

    #[test]
    fn resolves_custom_prefix_endpoint() {
        let endpoint = endpoint_for("http://127.0.0.1:8787", "/watchme/agent", "status").unwrap();
        assert_eq!(endpoint, "http://127.0.0.1:8787/watchme/agent/status");
    }

    #[test]
    fn rejects_unknown_message_type() {
        let result = endpoint_for("http://127.0.0.1:8787", "", "unknown");
        assert!(result.is_err());
    }
}
