use std::sync::Arc;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
mod jobs;
use tonk_shared_lib::{deserialize_struct, serialize_struct, Building, Location, Player, Game, GameStatus};
use tonk_shared_lib::redis_helper::*;
use crate::jobs::sync_graph::SyncGraph;
use crate::jobs::clock::Clock;
use crate::jobs::game_state::GameState;
use log::*;

// fn initialize_game_state() -> Result<(), Box<dyn std::error::Error>> {
//     // if we do this on the startup of state-service then better to just
//     // reset the state and wipe out all the old values
//     let mut con = create_redis_con()?;
//     let game = Game {
//         id: Uuid::new_v4().simple().to_string(),
//         status: GameStatus::Lobby,
//         time: None,
//         players: None
//     };
//     let bytes = serialize_struct(&game)?;
//     con.set("game", bytes)?;

//     Ok(())

//     // let result: Result<Vec<u8>, redis::RedisError> = con.get("game");
//     // match result {
//     //     Ok(_) => {
//     //         //nothing to do
//     //     }
//     //     Err(_) => {
//     //     }
//     // }
//     // Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let sched = JobScheduler::new().await?;

    // let sync_graph = SyncGraph::new().await?;
    // let shared_client = Arc::new(sync_graph).clone();

    // initialize_game_state()?;

    sched
        .add(Job::new_async("1/2 * * * * *", move |_, _| {
            Box::pin(async move {
                if let Ok(redis) = RedisHelper::init().await {
                    let sync_graph = SyncGraph::new(redis);
                    let r = sync_graph.run().await;
                    if r.is_err() {
                        info!("{}", r.err().unwrap());
                    }
                }
            })
        })?)
        .await?;

    sched
        .add(Job::new_async("*/1 * * * * *", move |_, _| {
            Box::pin(async move {
                if let Ok(redis) = RedisHelper::init().await {
                    let clock = Clock::new(redis);
                    let _ = clock.run().await;
                }
            })
        })?)
        .await?;

    sched
        .add(Job::new_async("1/3 * * * * *", move |_, _| {
            Box::pin(async move {
                if let Ok(redis) = RedisHelper::init().await {
                    let game_state = GameState::new(redis);
                    let _ = game_state.run().await;
                }
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
