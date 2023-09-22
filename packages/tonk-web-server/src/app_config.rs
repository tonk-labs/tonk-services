use actix_web::web;
use crate::handlers::{action, game, player, building};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::scope("/building")
            .service(
                web::resource("")
                    .route(web::post().to(building::post_building))
            )
    ).service(
        web::scope("/player")
            .service(
                web::scope("/{player_id}")
                    .service(
                        web::resource("")
                            .route(web::get().to(player::get_player))
                    )
            )
    ).service(
        web::scope("/game")
            .service(
                web::resource("")
                    .route(web::get().to(game::get_game))
                    .route(web::post().to(game::post_game))
                    .route(web::put().to(game::put_game))
            )
    ).service(
        web::scope("/action")
            .service(
                web::resource("")
                    .route(web::post().to(action::post_action))
            )
    );
}