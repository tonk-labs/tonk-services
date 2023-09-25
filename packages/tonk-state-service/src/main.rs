use std::sync::Arc;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
mod sync_client;
use crate::sync_client::SyncClient;
use uuid::Uuid;
use tonk_shared_lib::{deserialize_struct, serialize_struct, Building, Location, Player, Game, GameStatus};
use redis::Commands;

fn create_redis_con() -> redis::RedisResult<redis::Connection> {
    let redis_client = redis::Client::open("redis://0.0.0.0")?;
    redis_client.get_connection()
}

fn initialize_game_state() -> Result<(), Box<dyn std::error::Error>> {
    // if we do this on the startup of state-service then better to just
    // reset the state and wipe out all the old values
    let mut con = create_redis_con()?;
    let game = Game {
        id: Uuid::new_v4().simple().to_string(),
        status: GameStatus::Lobby,
        time: None,
        players: None
    };
    let bytes = serialize_struct(&game)?;
    con.set("game", bytes)?;

    Ok(())

    // let result: Result<Vec<u8>, redis::RedisError> = con.get("game");
    // match result {
    //     Ok(_) => {
    //         //nothing to do
    //     }
    //     Err(_) => {
    //     }
    // }
    // Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let sched = JobScheduler::new().await?;

    let shared_client = Arc::new(SyncClient::new());

    initialize_game_state()?;

    // Add basic cron job
    let closure_client = shared_client.clone();
    sched
        .add(Job::new_async("1/2 * * * * *", move |_uuid, _l| {
            let client = closure_client.clone();
            Box::pin(async move {
                // let mut con = match create_redis_con() {
                //     Ok(connection) => connection,
                //     Err(e) => {
                //         println!("Error getting connection: {}", e);
                //         return; // Return early since there was an error
                //     }
                // };

                // if let Err(e) = client.get_players(&mut con).await {
                //     println!("Error fetching players: {}", e);
                // }
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
