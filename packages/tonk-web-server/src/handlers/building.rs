use actix_web::{web, HttpResponse, Error};
use tonk_shared_lib::Building;
use tonk_shared_lib::redis_helper::*;

pub async fn post_building(_id: web::Json<Building>) -> Result<HttpResponse, Error> {
    let building = _id.0;
    let key = format!("building:{}", building.id);
    let redis = RedisHelper::init().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    let exists: Result<Building, _> = redis.get_key(&key).await;
    let result = redis.set_key(&key, &building).await;
    let fail_to_set = result.is_err();
    let mut resp = match result {
        Ok(_) => {
            HttpResponse::Ok().json(building)
        }
        Err(e) => {
            println!("{}", e.to_string());
            HttpResponse::InternalServerError().finish()
        }
    };
    if !fail_to_set {
        if exists.is_err() {
            if let Err(e) = redis.set_index("building:index", &key).await {
                println!("{}", e.to_string());
                resp = HttpResponse::InternalServerError().finish();
            }
        } 
    }
    Ok(resp)
}