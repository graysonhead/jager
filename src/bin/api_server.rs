#[macro_use]
extern crate rocket;
use jager::database;
use jager::stats_processing;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use serde::Serialize;
use std::error::Error;

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
        .register("/", catchers![not_found])
        .mount("/", routes![index])
        .mount("/", routes![get_character_stats])
}
