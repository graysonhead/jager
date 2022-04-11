#[macro_use]
extern crate rocket;
use backend::database::Db;
use backend::jager_redis;
use backend::stats_processing;
use rocket::request::Request;
use rocket::serde::json::Json;
use sea_orm_rocket::Connection;
use sea_orm_rocket::Database as SODatabase;
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
    character_name: String,
) -> Option<Json<stats_processing::CharacterStats>> {
    let db = conn.into_inner();
    let mut redis_conn = jager_redis::init_redis_connection().await;
    if let Some(conn) = &mut redis_conn {
        match jager_redis::check_cache_character_stats(conn, &character_name).await {
            Some(stats) => Some(Json(stats)),
            None => match stats_processing::get_character_stats(db, character_name.clone()).await {
                Ok(stats) => match stats {
                    Some(stats) => {
                        jager_redis::cache_character_stats(conn, &character_name, &stats).await;
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
    } else {
        match stats_processing::get_character_stats(db, character_name.clone()).await {
            Ok(stats) => match stats {
                Some(stats) => {
                    info!("Cache miss for {}", character_name);
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
        }
    }
}

#[launch]
fn rocket() -> _ {
    backend::logging::setup_logging();
    rocket::build()
        .attach(Db::init())
        .register("/", catchers![not_found])
        .mount("/", routes![index])
        .mount("/", routes![get_character_stats])
}
