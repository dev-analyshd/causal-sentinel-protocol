use anyhow::Result;
use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use serde_json::json;

use crate::behavioral_hasher::BehavioralHasher;

pub struct WebSocketServer {
    port: u16,
    hasher: Arc<RwLock<BehavioralHasher>>,
}

impl WebSocketServer {
    pub fn new(port: u16, hasher: Arc<RwLock<BehavioralHasher>>) -> Self {
        Self { port, hasher }
    }

    pub async fn run(&self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.port);
        info!("WebSocket server starting on ws://{}", addr);

        let hasher = self.hasher.clone();

        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(hasher);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(hasher): State<Arc<RwLock<BehavioralHasher>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, hasher))
}

async fn handle_socket(mut socket: WebSocket, hasher: Arc<RwLock<BehavioralHasher>>) {
    info!("WebSocket client connected");

    // Send initial state
    let state_msg = {
        let h = hasher.read().await;
        json!({
            "type": "state",
            "total_events": h.get_total_events(),
            "last_block": h.get_last_block(),
            "perceptual_entropy": h.get_perceptual_entropy(),
        })
        .to_string()
    };

    if socket.send(Message::Text(state_msg.into())).await.is_err() {
        return;
    }

    // Send periodic updates
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let update = {
            let h = hasher.read().await;
            json!({
                "type": "update",
                "total_events": h.get_total_events(),
                "last_block": h.get_last_block(),
                "perceptual_entropy": h.get_perceptual_entropy(),
            })
            .to_string()
        };

        if socket.send(Message::Text(update.into())).await.is_err() {
            error!("WebSocket send failed, closing connection");
            break;
        }
    }
}
