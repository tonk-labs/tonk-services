use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use log::*;
use num_bigint::BigInt;
use std::error::Error;
use redis::{Commands, ToRedisArgs, RedisResult};
use reqwest;
use tonk_shared_lib::{deserialize_struct, serialize_struct, Building, Location, Player, Game, GameStatus, redis_helper};
use tonk_shared_lib::redis_helper::*;

#[derive(GraphQLQuery, Debug)]
#[graphql(schema_path = "schema.json", query_path = "src/dsplayers.graphql")]
struct DSPlayers;

#[derive(GraphQLQuery, Debug)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/dsbuilding_tiles.graphql"
)]
struct DSBuildingTiles;

pub struct SyncGraph {
    client: reqwest::Client,
    redis: RedisHelper
}

impl SyncGraph {
    pub fn new(redis: RedisHelper) -> Self {
        Self {
            redis,
            client: reqwest::Client::new()
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let v = ds_players::Variables {
            game_id: "DOWNSTREAM".to_string(),
        };
        let res =
            post_graphql::<DSPlayers, _>(&self.client, "http://localhost:8080/query", v).await?;

        let mut players: Vec<Player> = Vec::new();
        for entry in &res.data.unwrap().game.state.nodes {
            if let (Some(player_resp), Some(location)) = (&entry.player, &entry.location) {
                if let Ok(player) = self.redis.get_key::<Player>(&format!("player:{}", player_resp.id)).await {
                    let location_coords = Location(
                        location.tile.coords[0].to_string(),
                        location.tile.coords[1].to_string(),
                        location.tile.coords[2].to_string(),
                        location.tile.coords[3].to_string(),
                    );

                    info!("{}", player.id);
                    info!(
                        "{} {} {} {}",
                        location.tile.coords[0],
                        location.tile.coords[1],
                        location.tile.coords[2],
                        location.tile.coords[3]
                    );

                    let updated_player = Player {
                        id: player.id,
                        nearby_buildings: None, //TODO: implement 
                        nearby_players: None, //TODO: implement 
                        location: Some(location_coords),
                        display_name: player.display_name,
                        secret_key: player.secret_key
                    };

                    players.push(updated_player);

                }
            }
        }

        for player in players {
            let player_key = format!("player:{}", player.id);
            let _: () = self.redis.set_key(&player_key, &player).await?;
        }
        Ok(())
    } 
}

//     // What to do for the buildings?
//     // Presumably, these aren't going to move
//     // That means we can just register them once upfront

//     // pub async fn get_buildings(&self, con: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error>> {
//     //     // let client = reqwest::Client::new();
//     //     let v = ds_building_tiles::Variables {
//     //         game_id: "DOWNSTREAM".to_string()
//     //     };
//     //     let res =
//     //     post_graphql::<DSBuildingTiles, _>(&self.client, "http://localhost:8080/query", v).await?;

//     //     for entry in &res.data.unwrap().game.state.nodes {
//     //         if let Some(building) = &entry.building {
//     //             // do some check here to make sure that it's a Depot building
//     //             // we'll need to know the ids somehow beforehand I suppose
//     //             // I suppose this will have to be set by some kind of admin?
//     //             let depot_id = BigInt::from(con.get("depot").unwrap_or("0"));
//     //             let tower_id = BigInt::from(con.get("tower").unwrap_or("0"));

//     //             if let Some(kind) = building {
//     //                 if kind.id == depot_id || {
//     //                     let location_coords = Location(
//     //                         entry.coords[0].to_string(),
//     //                         entry.coords[1].to_string(),
//     //                         entry.coords[2].to_string(),
//     //                         entry.coords[3].to_string()
//     //                     );

//     //                 }
//     //             }

//     //         }
//     //     }
//     //     // info!("{}", res.data.unwrap().game.id);
//     //     Ok(())
//     // }
// }
