use datamodels::esi_models::ESIKillmailRequest;
use chrono::{Duration, NaiveDate};
use futures::{stream, StreamExt};
use serde_json::{Map, Value};
use std::convert::TryInto;

const ZKILL_URL: &str = "https://zkillboard.com/api";

#[derive(Debug)]
pub enum ZKillError {
    APIError(reqwest::Error),
    Parse(serde_json::Error),
}

impl From<reqwest::Error> for ZKillError {
    fn from(err: reqwest::Error) -> ZKillError {
        ZKillError::APIError(err)
    }
}

impl From<serde_json::Error> for ZKillError {
    fn from(err: serde_json::Error) -> ZKillError {
        ZKillError::Parse(err)
    }
}

fn get_url(path: String) -> String {
    format!("{}/{}", ZKILL_URL, path)
}

pub async fn get_killmail_requests_from_date(
    date: NaiveDate,
) -> Result<Vec<ESIKillmailRequest>, ZKillError> {
    let timestamp = date.format("%Y%m%d");
    let url = get_url(format!("history/{}.json", timestamp));
    info!("Fetching history from {}", url);
    let response = reqwest::get(url).await?;
    let result_str: String = response.text().await?;
    let value: Map<String, Value> = serde_json::from_str(&result_str)?;
    let requests = value
        .into_iter()
        .map(|(k, v)| ESIKillmailRequest {
            id: k,
            hash: v.as_str().unwrap().to_string(),
        })
        .collect::<Vec<ESIKillmailRequest>>();
    Ok(requests)
}

pub async fn get_history_records(dates: Vec<NaiveDate>) -> Vec<ESIKillmailRequest> {
    let mut records: Vec<ESIKillmailRequest> = Vec::new();
    let mut bodies = stream::iter(dates)
        .map(|date| async move { get_killmail_requests_from_date(date).await })
        .buffer_unordered(10);
    while let Some(record) = bodies.next().await {
        match record {
            Ok(rec) => {
                let mut record_cln = rec.clone();
                records.append(&mut record_cln);
            }
            Err(e) => {
                error!("Couldn't process zkill records: {:?}", e);
            }
        }
    }
    records
}

pub fn get_dates(start_date: chrono::NaiveDate, days_to_fetch: u64) -> Vec<NaiveDate> {
    let mut dates: Vec<NaiveDate> = Vec::new();
    let current_date = start_date;
    dates.push(current_date);
    for days in 1..days_to_fetch {
        let new_date = current_date - Duration::days(days.try_into().unwrap());
        dates.push(new_date);
    }
    dates
}
