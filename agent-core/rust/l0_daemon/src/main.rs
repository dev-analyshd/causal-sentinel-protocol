use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, error};

mod casper_stream;
mod behavioral_hasher;
mod websocket_server;
mod config;

use config::DaemonConfig;
use casper_stream::CasperEventStream;
use behavioral_hasher::BehavioralHasher;
use websocket_server::WebSocketServer;

/// L0 Daemon: Casper event streaming, behavioral hashing (SHA3-256 dual)
///
/// The daemon connects to Casper nodes via WebSocket, ingests all relevant
/// events (transfers, deploys, consensus messages), and computes dual
/// SHA3-256 behavioral hashes that feed into the coherence engine.

#[derive(Parser, Debug)]
#[command(name = "l0-daemon")]
#[command(about = "Causal Sentinel L0 Event Daemon")]
struct Args {
    /// Config file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Casper node RPC endpoint
    #[arg(short, long)]
    node: Option<String>,

    /// WebSocket port for downstream consumers
    #[arg(short, long, default_value = "9001")]
    port: u16,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if args.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🔱 Causal Sentinel L0 Daemon starting...");

    // Load config
    let config = DaemonConfig::load(&args.config).await?;
    let node_url = args.node.unwrap_or(config.node_url);

    info!("Connecting to Casper node: {}", node_url);

    // Shared state
    let hasher = Arc::new(RwLock::new(BehavioralHasher::new()));
    let (event_tx, mut event_rx) = mpsc::channel::<CasperEvent>(10000);

    // Start Casper event stream
    let stream = CasperEventStream::new(&node_url, event_tx.clone());
    let stream_handle = tokio::spawn(async move {
        if let Err(e) = stream.run().await {
            error!("Event stream error: {}", e);
        }
    });

    // Start WebSocket server for downstream consumers
    let ws_server = WebSocketServer::new(args.port, hasher.clone());
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = ws_server.run().await {
            error!("WebSocket server error: {}", e);
        }
    });

    // Main event processing loop
    info!("Event processing loop started");

    while let Some(event) = event_rx.recv().await {
        match event {
            CasperEvent::DeployAccepted { deploy_hash, account, timestamp } => {
                let mut h = hasher.write().await;
                h.process_deploy(&deploy_hash, &account, timestamp);
                drop(h);

                if args.debug {
                    info!("Deploy accepted: {} from {}", hex::encode(deploy_hash), account);
                }
            },
            CasperEvent::BlockAdded { block_hash, height, era_id, timestamp } => {
                let mut h = hasher.write().await;
                h.process_block(&block_hash, height, era_id, timestamp);
                drop(h);

                info!("Block #{} added (era {})", height, era_id);
            },
            CasperEvent::Transfer { from, to, amount, deploy_hash } => {
                let mut h = hasher.write().await;
                h.process_transfer(&from, &to, amount, &deploy_hash);
                drop(h);
            },
            CasperEvent::ConsensusMessage { public_key, era_id, message_type } => {
                let mut h = hasher.write().await;
                h.process_consensus(&public_key, era_id, &message_type);
                drop(h);
            },
            CasperEvent::Step { era_id, execution_effects } => {
                let mut h = hasher.write().await;
                h.process_step(era_id, &execution_effects);
                drop(h);
            },
        }
    }

    // Graceful shutdown
    warn!("Event stream closed, shutting down...");
    stream_handle.abort();
    ws_handle.abort();

    info!("L0 Daemon shutdown complete");
    Ok(())
}

#[derive(Debug, Clone)]
pub enum CasperEvent {
    DeployAccepted {
        deploy_hash: [u8; 32],
        account: String,
        timestamp: u64,
    },
    BlockAdded {
        block_hash: [u8; 32],
        height: u64,
        era_id: u64,
        timestamp: u64,
    },
    Transfer {
        from: String,
        to: String,
        amount: u64,
        deploy_hash: [u8; 32],
    },
    ConsensusMessage {
        public_key: String,
        era_id: u64,
        message_type: String,
    },
    Step {
        era_id: u64,
        execution_effects: Vec<String>,
    },
}
