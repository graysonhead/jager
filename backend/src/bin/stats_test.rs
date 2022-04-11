#[macro_use]
extern crate log;

extern crate pbr;

use backend::database::establish_connection;
use backend::stats_processing;

#[tokio::main]
async fn main() {
    backend::logging::setup_logging();
    info!("Establishing connection");
    let db = establish_connection().await.unwrap();
    let res = stats_processing::get_character_stats(&db, "Darkside 34".to_string()).await;
    println!("{:?}", res);
}
