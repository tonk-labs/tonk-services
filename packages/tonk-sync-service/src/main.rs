use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use log::*;
use reqwest;
use num_bigint::{BigInt};

#[derive(GraphQLQuery, Debug)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/players.graphql",
)]
struct Players;

async fn get_players() -> Result<(),Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let v = players::Variables {
        game_id: "DOWNSTREAM".to_string()
    };
    let res =
    post_graphql::<Players, _>(&client, "http://localhost:8080/query", v).await?;

    info!("{}", res.data.unwrap().game.id);
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut sched = JobScheduler::new().await?;

    // Add basic cron job
    sched.add(
        Job::new_async("1/2 * * * * *", |_uuid, _l| {
            Box::pin(async move {
                if let Err(e) = get_players().await {
                    println!("Error fetching players: {}", e);
                }
            })
        })?
    ).await?;

    // Start the scheduler
    sched.start().await?;

    // Wait while the jobs run
    tokio::time::sleep(Duration::from_secs(100)).await;

    Ok(())

}