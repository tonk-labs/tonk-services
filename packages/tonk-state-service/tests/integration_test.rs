mod common;

#[tokio::test]
async fn run_test_services() -> Result<(), Box<dyn std::error::Error>>  {
    // using common code.
    let _ = common::setup_services().await;
    Ok(())
}