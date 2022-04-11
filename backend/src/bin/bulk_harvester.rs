#[macro_use]
extern crate log;

extern crate pbr;

use backend::database::establish_connection;
use backend::killmail_processing;
use backend::zkill;
use chrono::NaiveDate;
use chrono::Utc;
use pbr::ProgressBar;

#[tokio::main]
async fn main() {
    backend::logging::setup_logging();
    info!("Establishing connection");
    let db = establish_connection().await.unwrap();
    let now: NaiveDate = Utc::today().naive_utc();
    let dates = zkill::get_dates(now, 90);
    let requests = zkill::get_history_records(dates).await;
    let mut pb = ProgressBar::new(requests.len().try_into().unwrap());
    println!("Fetching {} killmails", requests.len());
    killmail_processing::process_killmails(&db, requests, &mut pb)
        .await
        .unwrap();
}
