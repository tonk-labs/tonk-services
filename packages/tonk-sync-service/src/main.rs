use std::sync::Arc;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
mod sync_client;
use crate::sync_client::SyncClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let sched = JobScheduler::new().await?;

    let shared_client = Arc::new(SyncClient::new());

    // Add basic cron job
    let closure_client = shared_client.clone();
    sched
        .add(Job::new_async("1/2 * * * * *", move |_uuid, _l| {
            let client = closure_client.clone();
            Box::pin(async move {
                let redis_client = match redis::Client::open("redis://0.0.0.0") {
                    Ok(client) => client,
                    Err(e) => {
                        println!("Error creating Redis client: {}", e);
                        return; // Return early since there was an error
                    }
                };

                let mut con = match redis_client.get_connection() {
                    Ok(connection) => connection,
                    Err(e) => {
                        println!("Error getting connection: {}", e);
                        return; // Return early since there was an error
                    }
                };

                if let Err(e) = client.get_players(&mut con).await {
                    println!("Error fetching players: {}", e);
                }
                // if let Err(e) = client.get_buildings(&mut con).await {
                //     println!("Error fetching players: {}", e);
                // }
            })
        })?)
        .await?;

    // Start the scheduler
    sched.start().await?;

    // Wait while the jobs run
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
