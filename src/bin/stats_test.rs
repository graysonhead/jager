#[macro_use]
extern crate log;

extern crate pbr;

use chrono::NaiveDate;
use chrono::Utc;
use jager::database::establish_connection;
use jager::stats_processing;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Establishing connection");
    let db = establish_connection().await.unwrap();
    let res = stats_processing::get_character_stats(db, "Darkside 34".to_string()).await;
    println!("{:?}", res);
}
