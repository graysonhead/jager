#[macro_use]
extern crate rocket;
use jager::database;
use jager::stats_processing;
use rocket::http::{ContentType, Status};
use rocket::response::Response;
use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize)]
struct SrdError {
    err: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/character_stats/<character_name>")]
async fn get_character_stats(
    character_name: String,
) -> Option<Json<stats_processing::CharacterStats>> {
    let db = database::establish_connection().await.unwrap();
    let result = stats_processing::get_character_stats(db, character_name).await;
    match result {
        Ok(option) => match option {
            Some(stats) => Some(Json(stats)),
            None => None,
        },
        Err(e) => None,
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![get_character_stats])
}
