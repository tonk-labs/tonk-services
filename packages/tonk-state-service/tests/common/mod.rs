use tonk_state_service::run_test;

pub async fn setup_services() -> Result<(), Box<dyn std::error::Error>>  {
    // some setup code, like creating required files/directories, starting
    // servers, etc.
    run_test().await
}