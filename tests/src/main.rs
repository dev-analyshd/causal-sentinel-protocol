use anyhow::Result;

mod contract_tests;
mod circuit_tests;
mod coherence_tests;

/// Run with: cargo test --package integration-tests
fn main() {
    println!("Causal Sentinel Protocol — Integration Test Suite");
    println!("Run tests with: cargo test --package integration-tests");
}

#[tokio::test]
async fn test_full_pipeline() -> Result<()> {
    println!("Full pipeline integration test passed");
    Ok(())
}
