use anyhow::Result;
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
        // Connect to Casper node's event stream API
        let url = format!("{}/events/deploys", self.node_url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to connect: {}", response.status()));
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
                let deploy_hash = hex::decode(
                    event["deploy_hash"].as_str().unwrap_or("")
                )?;
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&deploy_hash);

                let casper_event = CasperEvent::DeployAccepted {
                    deploy_hash: hash,
                    account: event["account"].as_str().unwrap_or("").to_string(),
                    timestamp: event["timestamp"].as_u64().unwrap_or(0),
                };

                self.event_tx.send(casper_event).await?;
            },
            "BlockAdded" => {
                let block_hash = hex::decode(
                    event["block_hash"].as_str().unwrap_or("")
                )?;
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&block_hash);

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
                    to: event["to"].as_str().unwrap_or("").to_string(),
                    amount: event["amount"].as_u64().unwrap_or(0),
                    deploy_hash: [0u8; 32], // Simplified
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
