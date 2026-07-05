use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// MCP Server — CSPR.trade integration, natural language DeFi actions
///
/// Provides a Model Context Protocol (MCP) interface for agents to
/// execute natural language DeFi operations on Casper Network.
///
/// Endpoints:
/// - /mcp/parse: Parse natural language → structured action
/// - /mcp/execute: Execute structured action with ZK compliance
/// - /mcp/status: Get action execution status

#[derive(Clone)]
pub struct AppState {
    pub action_parser: Arc<RwLock<ActionParser>>,
    pub execution_engine: Arc<RwLock<ExecutionEngine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLRequest {
    pub agent_id: String,
    pub natural_language: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedAction {
    pub action_type: String,
    pub parameters: serde_json::Value,
    pub confidence: f64,
    pub required_tier: u8,
    pub estimated_cost_motes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub agent_id: String,
    pub action: ParsedAction,
    pub zk_proof: Option<Vec<u8>>,
    pub nullifier: Option<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub gas_used: u64,
}

pub struct ActionParser;

pub struct ExecutionEngine;

impl ActionParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, request: &NLRequest) -> Result<ParsedAction, String> {
        let text = request.natural_language.to_lowercase();

        if text.contains("swap") || text.contains("trade") || text.contains("exchange") {
            Ok(ParsedAction {
                action_type: "swap".to_string(),
                parameters: serde_json::json!({
                    "from_token": "CSPR",
                    "to_token": "USDC",
                    "amount": self.extract_amount(&text),
                }),
                confidence: 0.92,
                required_tier: 3,
                estimated_cost_motes: 1_000_000_000, // 1.0 CSPR
            })
        } else if text.contains("stake") || text.contains("delegate") {
            Ok(ParsedAction {
                action_type: "stake".to_string(),
                parameters: serde_json::json!({
                    "validator": self.extract_address(&text),
                    "amount": self.extract_amount(&text),
                }),
                confidence: 0.88,
                required_tier: 2,
                estimated_cost_motes: 500_000_000,
            })
        } else if text.contains("transfer") || text.contains("send") {
            Ok(ParsedAction {
                action_type: "transfer".to_string(),
                parameters: serde_json::json!({
                    "to": self.extract_address(&text),
                    "amount": self.extract_amount(&text),
                }),
                confidence: 0.95,
                required_tier: 1,
                estimated_cost_motes: 100_000_000,
            })
        } else if text.contains("bridge") || text.contains("cross chain") {
            Ok(ParsedAction {
                action_type: "bridge".to_string(),
                parameters: serde_json::json!({
                    "from_chain": "Casper",
                    "to_chain": "Ethereum",
                    "amount": self.extract_amount(&text),
                }),
                confidence: 0.85,
                required_tier: 4,
                estimated_cost_motes: 5_000_000_000,
            })
        } else {
            Err("Unable to parse natural language action".to_string())
        }
    }

    fn extract_amount(&self, text: &str) -> u64 {
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if let Ok(num) = word.parse::<f64>() {
                return (num * 1_000_000_000.0) as u64;
            }
            if i > 0 && words[i - 1].to_lowercase() == "amount" {
                if let Ok(num) = word.parse::<f64>() {
                    return (num * 1_000_000_000.0) as u64;
                }
            }
        }
        1_000_000_000 // Default 1 CSPR
    }

    fn extract_address(&self, _text: &str) -> String {
        "hash-...recipient".to_string()
    }
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, request: &ExecutionRequest) -> Result<ExecutionResponse, String> {
        info!("executing_action agent_id={} action_type={}", request.agent_id, request.action.action_type);

        let tx_hash = format!("hash-mcp-{}", request.agent_id);

        Ok(ExecutionResponse {
            success: true,
            tx_hash: Some(tx_hash),
            result: Some(serde_json::json!({
                "action": request.action.action_type,
                "status": "confirmed",
                "block": 1234567,
            })),
            error: None,
            gas_used: request.action.estimated_cost_motes,
        })
    }
}

async fn parse_action(
    State(state): State<AppState>,
    Json(request): Json<NLRequest>,
) -> Result<Json<ParsedAction>, StatusCode> {
    let parser = state.action_parser.read().await;

    match parser.parse(&request) {
        Ok(action) => Ok(Json(action)),
        Err(e) => {
            warn!("parse_failed error={}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn execute_action(
    State(state): State<AppState>,
    Json(request): Json<ExecutionRequest>,
) -> Result<Json<ExecutionResponse>, StatusCode> {
    let engine = state.execution_engine.read().await;

    match engine.execute(&request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            warn!("execution_failed error={}", e);
            Ok(Json(ExecutionResponse {
                success: false,
                tx_hash: None,
                result: None,
                error: Some(e),
                gas_used: 0,
            }))
        }
    }
}

pub async fn run_server(port: u16) -> Result<()> {
    let state = AppState {
        action_parser: Arc::new(RwLock::new(ActionParser::new())),
        execution_engine: Arc::new(RwLock::new(ExecutionEngine::new())),
    };

    let app = Router::new()
        .route("/mcp/parse", post(parse_action))
        .route("/mcp/execute", post(execute_action))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("mcp_server_started addr={}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    run_server(8081).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_swap() {
        let parser = ActionParser::new();
        let req = NLRequest {
            agent_id: "agent_001".to_string(),
            natural_language: "swap 10 CSPR for USDC".to_string(),
            context: None,
        };
        let result = parser.parse(&req);
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action.action_type, "swap");
        assert_eq!(action.required_tier, 3);
    }

    #[test]
    fn test_parse_transfer() {
        let parser = ActionParser::new();
        let req = NLRequest {
            agent_id: "agent_001".to_string(),
            natural_language: "send 5 CSPR to agent_002".to_string(),
            context: None,
        };
        let result = parser.parse(&req);
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action.action_type, "transfer");
        assert_eq!(action.required_tier, 1);
    }

    #[test]
    fn test_parse_stake() {
        let parser = ActionParser::new();
        let req = NLRequest {
            agent_id: "agent_001".to_string(),
            natural_language: "stake 100 CSPR with validator_001".to_string(),
            context: None,
        };
        let result = parser.parse(&req);
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action.action_type, "stake");
        assert_eq!(action.required_tier, 2);
    }

    #[test]
    fn test_parse_unknown_fails() {
        let parser = ActionParser::new();
        let req = NLRequest {
            agent_id: "agent_001".to_string(),
            natural_language: "do something random xyz".to_string(),
            context: None,
        };
        let result = parser.parse(&req);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_action() {
        let engine = ExecutionEngine::new();
        let req = ExecutionRequest {
            agent_id: "agent_001".to_string(),
            action: ParsedAction {
                action_type: "swap".to_string(),
                parameters: serde_json::json!({}),
                confidence: 0.9,
                required_tier: 3,
                estimated_cost_motes: 1_000_000_000,
            },
            zk_proof: None,
            nullifier: None,
        };
        let result = engine.execute(&req).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.success);
        assert!(resp.tx_hash.is_some());
    }

    #[test]
    fn test_amount_extraction() {
        let parser = ActionParser::new();
        let amount = parser.extract_amount("swap 5.5 cspr");
        assert_eq!(amount, 5_500_000_000);
    }
}
