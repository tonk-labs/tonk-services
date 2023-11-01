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
use std::env;
use dotenv::dotenv;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let sched = JobScheduler::new().await?;

    // let sync_graph = SyncGraph::new().await?;
    // let shared_client = Arc::new(sync_graph).clone();

    // initialize_game_state()?;
    match env::var("TONK_SERVICES_STAGE") {
        Ok(stage) => {
            println!("Starting up tonk-state-service in stage: {}", stage);
            dotenv::from_filename(".env.production").ok();
        }
        Err(_) => {
            dotenv::from_filename(".env.local").ok();
        }
    }

    sched
        .add(Job::new_async("1/2 * * * * *", move |_, _| {
            Box::pin(async move {
                if let Ok(redis) = RedisHelper::init().await {
                    let sync_graph = SyncGraph::new(redis);
                    let r = sync_graph.run().await;
                    if r.is_err() {
                        error!("{}", r.err().unwrap());
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
                    let r = clock.run().await;
                    if r.is_err() {
                        error!("{:?}", r.err().unwrap());
                    }
                }
            })
        })?)
        .await?;

    sched
        .add(Job::new_async("1/3 * * * * *", move |_, _| {
            Box::pin(async move {
                if let Ok(redis) = RedisHelper::init().await {
                    let game_state = GameState::new(redis);
                    let r = game_state.run().await;
                    if r.is_err() {
                        error!("{:?}", r.err().unwrap());
                    }
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
