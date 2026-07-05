use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error, debug};

use crate::CasperEvent;

pub struct CasperEventStream {
    node_url: String,
    event_tx: Sender<CasperEvent>,
    client: Client,
}

impl CasperEventStream {
    pub fn new(node_url: &str, event_tx: Sender<CasperEvent>) -> Self {
        Self {
            node_url: node_url.to_string(),
            event_tx,
            client: Client::new(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting Casper event stream from {}", self.node_url);

        loop {
            match self.connect_and_stream().await {
                Ok(_) => {
                    warn!("Event stream closed, reconnecting...");
                },
                Err(e) => {
                    error!("Event stream error: {}, reconnecting in 5s...", e);
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_stream(&self) -> Result<()> {
        let url = format!("{}/events/deploys", self.node_url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to connect: {}", response.status()));
        }

        let body = response.text().await?;
        let events: Vec<Value> = serde_json::from_str(&body)?;

        for event in events {
            self.parse_and_forward(event).await?;
        }

        Ok(())
    }

    async fn parse_and_forward(&self, event: Value) -> Result<()> {
        let event_type = event["event_type"].as_str().unwrap_or("unknown");

        match event_type {
            "DeployAccepted" => {
                let hash = parse_hash32(&event, "deploy_hash")?;
                let casper_event = CasperEvent::DeployAccepted {
                    deploy_hash: hash,
                    account: event["account"].as_str().unwrap_or("").to_string(),
                    timestamp: event["timestamp"].as_u64().unwrap_or(0),
                };
                self.event_tx.send(casper_event).await?;
            },
            "BlockAdded" => {
                let hash = parse_hash32(&event, "block_hash")?;
                let casper_event = CasperEvent::BlockAdded {
                    block_hash: hash,
                    height: event["height"].as_u64().unwrap_or(0),
                    era_id: event["era_id"].as_u64().unwrap_or(0),
                    timestamp: event["timestamp"].as_u64().unwrap_or(0),
                };
                self.event_tx.send(casper_event).await?;
            },
            "Transfer" => {
                let casper_event = CasperEvent::Transfer {
                    from: event["from"].as_str().unwrap_or("").to_string(),
                    to:   event["to"].as_str().unwrap_or("").to_string(),
                    amount: event["amount"].as_u64().unwrap_or(0),
                    deploy_hash: [0u8; 32],
                };
                self.event_tx.send(casper_event).await?;
            },
            _ => {
                debug!("Unknown event type: {}", event_type);
            }
        }

        Ok(())
    }
}

/// Decode a hex-encoded 32-byte hash from a JSON field.
/// Returns `Err` (logged, does not panic) if the field is missing, non-hex,
/// or not exactly 32 bytes after decoding.
fn parse_hash32(event: &Value, field: &str) -> Result<[u8; 32]> {
    let hex_str = event[field]
        .as_str()
        .ok_or_else(|| anyhow!("field '{}' missing or not a string", field))?;

    let bytes = hex::decode(hex_str)
        .map_err(|e| anyhow!("field '{}' is not valid hex: {}", field, e))?;

    if bytes.len() != 32 {
        return Err(anyhow!(
            "field '{}' decoded to {} bytes, expected exactly 32",
            field, bytes.len()
        ));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);  // safe: length verified above
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_hash32_valid() {
        let event = json!({ "deploy_hash": "a".repeat(64) });
        let result = parse_hash32(&event, "deploy_hash");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_hash32_too_short() {
        let event = json!({ "deploy_hash": "deadbeef" }); // 4 bytes, not 32
        let result = parse_hash32(&event, "deploy_hash");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected exactly 32"));
    }

    #[test]
    fn test_parse_hash32_invalid_hex() {
        let event = json!({ "deploy_hash": "gggg" });
        let result = parse_hash32(&event, "deploy_hash");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not valid hex"));
    }

    #[test]
    fn test_parse_hash32_missing_field() {
        let event = json!({});
        let result = parse_hash32(&event, "deploy_hash");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing"));
    }
}
