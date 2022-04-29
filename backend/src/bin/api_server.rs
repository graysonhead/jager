#[macro_use]
extern crate rocket;
use backend::database::Db;
use backend::jager_redis;
use backend::stats_processing;
use bb8_redis::bb8::Pool;
use bb8_redis::RedisConnectionManager;
use rocket::fairing::{AdHoc};
use rocket::request::Request;
use rocket::serde::json::Json;
use rocket::State;
use sea_orm_rocket::Connection;
use sea_orm_rocket::Database as SODatabase;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ErrorMessage {
    message: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[catch(404)]
fn not_found(_req: &Request) -> Json<ErrorMessage> {
    Json(ErrorMessage {
        message: "The resource you were looking for doesn't exist".to_string(),
    })
}

#[get("/character_stats/<character_name>")]
async fn get_character_stats(
    conn: Connection<'_, Db>,
    redis_pool: &State<Pool<RedisConnectionManager>>,
    character_name: String,
) -> Option<Json<stats_processing::CharacterStats>> {
    let db = conn.into_inner();
    let mut redis_conn = redis_pool.clone().get().await.unwrap();
    match jager_redis::check_cache_character_stats(&mut redis_conn, &character_name).await {
        Some(stats) => Some(Json(stats)),
        None => match stats_processing::get_character_stats(db, character_name.clone()).await {
            Ok(stats) => match stats {
                Some(stats) => {
                    jager_redis::cache_character_stats(&mut redis_conn, &character_name, &stats)
                        .await;
                    Some(Json(stats))
                }
                None => None,
            },
            Err(e) => {
                error!(
                    "Failed to fetch character stats from db for {}: {:?}",
                    character_name, e
                );
                None
            }
        },
    }
}

#[derive(Deserialize)]
struct RedisConfig {
    redis_url: String,
}

#[launch]
async fn rocket() -> _ {
    use figment::{
        providers::{Format, Toml},
    };

    let figment = rocket::Config::figment()
        .merge(rocket::Config::default())
        .merge(Toml::file("Rocket.toml").nested());
    let redis_config: RedisConfig = figment.extract().unwrap();
    let pool = jager_redis::get_redis_pool(redis_config.redis_url).await;
    backend::logging::setup_logging();
    rocket::custom(figment)
        .attach(Db::init())
        .attach(AdHoc::config::<RedisConfig>())
        .register("/", catchers![not_found])
        .manage(pool)
        .mount("/", routes![index])
        .mount("/", routes![get_character_stats])
}
