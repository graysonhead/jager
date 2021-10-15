use crate::esi_models::*;
use futures::{stream, StreamExt};
use reqwest;
use std::str;
use tokio;
use tokio::time::{sleep, Duration};

const ESI_URL: &str = "https://esi.evetech.net";
const ESI_VERSION: &str = "latest";
const DATASOURCE: &str = "tranquility";
const RETRY_COUNT: u64 = 10;
const CONCURRENCY: usize = 100;

pub fn get_uri(path: String) -> String {
    format!(
        "{}/{}/{}/?datasource={}",
        ESI_URL, ESI_VERSION, path, DATASOURCE
    )
}

#[derive(Debug)]
pub enum EsiError {
    ApiError(reqwest::Error),
    Parse(serde_json::Error),
}

impl From<reqwest::Error> for EsiError {
    fn from(err: reqwest::Error) -> EsiError {
        EsiError::ApiError(err)
    }
}

impl From<serde_json::Error> for EsiError {
    fn from(err: serde_json::Error) -> EsiError {
        EsiError::Parse(err)
    }
}

pub async fn get_type_list() -> Result<Vec<u64>, EsiError> {
    let results = get_paginated_esi_list_results("universe/types".to_string()).await?;
    Ok(results)
}

pub async fn get_group_list() -> Result<Vec<u64>, EsiError> {
    let results = get_paginated_esi_list_results("universe/groups".to_string()).await?;
    Ok(results)
}

pub async fn get_category_list() -> Result<Vec<u64>, EsiError> {
    let results = get_paginated_esi_list_results("universe/categories".to_string()).await?;
    Ok(results)
}

pub async fn get_paginated_esi_list_results(uri_path: String) -> Result<Vec<u64>, EsiError> {
    let request_uri = get_uri(uri_path.to_string());
    info!("Sent request to {}", request_uri);
    let response = reqwest::get(&request_uri).await?;
    let headers = response.headers();
    let mut last_page: i32 = 1;
    if let Some(pages) = headers.get("x-pages") {
        let header_str = str::from_utf8(pages.as_bytes()).unwrap();
        last_page = header_str.parse::<i32>().unwrap();
    }
    let mut results: Vec<u64> = response.json().await?;
    if last_page > 1 {
        let urls: Vec<String> = (2..(last_page + 1))
            .map(|page_num| {
                format!(
                    "{}{}",
                    get_uri(uri_path.to_string()),
                    format!("&page={}", page_num.to_string())
                )
            })
            .collect();
        let mut bodies = stream::iter(urls)
            .map(|url| async move {
                info!("Sending request to {}", url);
                let res: Vec<u64> = reqwest::get(url).await.unwrap().json().await.unwrap();
                res
            })
            .buffer_unordered(CONCURRENCY);
        while let Some(mut item) = bodies.next().await {
            results.append(&mut item);
        }
    }
    Ok(results)
}

pub async fn get_esi_categories(category_ids: Vec<u64>) -> Vec<ESICategory> {
    let mut results: Vec<ESICategory> = Vec::new();
    let request_urls: Vec<String> = category_ids
        .into_iter()
        .map(|id| get_uri(format!("universe/categories/{}", id)))
        .collect();
    let mut bodies = stream::iter(request_urls)
        .map(|url| async move {
            info!("Sending request to {}", url);
            let res: Result<String, EsiError> = get_text_retry(url).await;
            res
        })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(text) => {
                let object: Result<ESICategory, serde_json::Error> = serde_json::from_str(&text);
                match object {
                    Ok(object) => results.push(object),
                    Err(e) => error!("Couldn't deserialize category: {}", e),
                }
            }
            Err(e) => error!("Couldn't fetch category {:?}", e),
        }
    }
    results
}

pub async fn get_esi_groups(category_ids: Vec<u64>) -> Vec<ESIGroup> {
    let mut results: Vec<ESIGroup> = Vec::new();
    let request_urls: Vec<String> = category_ids
        .into_iter()
        .map(|id| get_uri(format!("universe/groups/{}", id)))
        .collect();
    let mut bodies = stream::iter(request_urls)
        .map(|url| async move {
            info!("Sending request to {}", url);
            let res: Result<String, EsiError> = get_text_retry(url).await;
            res
        })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(text) => {
                let object: Result<ESIGroup, serde_json::Error> = serde_json::from_str(&text);
                match object {
                    Ok(object) => results.push(object),
                    Err(e) => error!("Couldn't deserialize category: {}", e),
                }
            }
            Err(e) => error!("Couldn't fetch category: {:?}", e),
        }
    }
    results
}

pub async fn get_esi_types(type_ids: Vec<u64>) -> Vec<ESIType> {
    let mut results: Vec<ESIType> = Vec::new();
    let request_urls: Vec<String> = type_ids
        .into_iter()
        .map(|id| get_uri(format!("universe/types/{}", id)))
        .collect();
    let mut bodies = stream::iter(request_urls)
        .map(|url| async move {
            info!("Sending request to {}", url);
            let res: Result<String, EsiError> = get_text_retry(url).await;
            res
        })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(text) => {
                let object: Result<ESIType, serde_json::Error> = serde_json::from_str(&text);
                match object {
                    Ok(object) => results.push(object),
                    Err(e) => error!("Couldn't deserialize type: {}", e),
                }
            }
            Err(e) => error!("Couldn't fetch category: {:?}", e),
        }
    }
    results
}

pub async fn get_character(
    character_id: u64,
) -> Result<crate::entity::character_public_info::ActiveModel, EsiError> {
    let req_uri = get_uri(format!("characters/{}", character_id));
    let res: Result<String, EsiError> = get_text_retry(req_uri).await;
    let result = match res {
        Ok(text) => {
            let object: Result<EsiCharacterPublicInfo, serde_json::Error> =
                serde_json::from_str(&text);
            match object {
                Ok(object) => {
                    let insertable = crate::entity::character_public_info::ActiveModel::from_esi(
                        character_id,
                        &object,
                    );
                    Ok(insertable)
                }
                Err(e) => {
                    error!("Couldn't deserialize type: {}", e);
                    Err(EsiError::from(e))
                }
            }
        }
        Err(e) => {
            error!("Couldn't fetch character: {:?}", e);
            Err(e)
        }
    };
    result
}

pub async fn get_characters(
    character_ids: Vec<u64>,
) -> Result<Vec<crate::entity::character_public_info::ActiveModel>, EsiError> {
    let mut results: Vec<crate::entity::character_public_info::ActiveModel> = Vec::new();
    let mut err: Option<EsiError> = Option::None;
    let request_urls: Vec<u64> = character_ids;
    let mut bodies = stream::iter(request_urls)
        .map(|id| async move { get_character(id).await })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(char) => {
                results.push(char);
            }
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(results)
    }
}

pub async fn get_alliances(
    alliance_ids: Vec<u64>,
) -> Result<Vec<crate::entity::alliances::ActiveModel>, EsiError> {
    let mut results: Vec<crate::entity::alliances::ActiveModel> = Vec::new();
    let mut err: Option<EsiError> = Option::None;
    let request_urls: Vec<u64> = alliance_ids;
    let mut bodies = stream::iter(request_urls)
        .map(|id| async move { get_alliance(id).await })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(alliance) => {
                results.push(alliance);
            }
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(results)
    }
}

pub async fn get_corporations(
    corporation_ids: Vec<u64>,
) -> Result<Vec<crate::entity::corporations::ActiveModel>, EsiError> {
    let mut results: Vec<crate::entity::corporations::ActiveModel> = Vec::new();
    let mut err: Option<EsiError> = Option::None;
    let request_urls: Vec<u64> = corporation_ids;
    let mut bodies = stream::iter(request_urls)
        .map(|id| async move { get_corporation(id).await })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(corporation) => {
                results.push(corporation);
            }
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(results)
    }
}

pub async fn get_alliance(
    alliance_id: u64,
) -> Result<crate::entity::alliances::ActiveModel, EsiError> {
    let req_uri = get_uri(format!("alliances/{}", alliance_id));
    let res: Result<String, EsiError> = get_text_retry(req_uri).await;
    let result = match res {
        Ok(text) => {
            let object: Result<ESIAlliance, serde_json::Error> = serde_json::from_str(&text);
            match object {
                Ok(object) => {
                    let insertable =
                        crate::entity::alliances::ActiveModel::from_esi(alliance_id, object);
                    Ok(insertable)
                }
                Err(e) => {
                    error!("Couldn't deserialize alliance {}: {:?}", alliance_id, e);
                    Err(EsiError::from(e))
                }
            }
        }
        Err(e) => {
            error!("Couldn't fetch alliance {}: {:?}", alliance_id, e);
            Err(e)
        }
    };
    result
}

pub async fn get_corporation(
    corporation_id: u64,
) -> Result<crate::entity::corporations::ActiveModel, EsiError> {
    let req_uri = get_uri(format!("corporations/{}", corporation_id));
    let res: Result<String, EsiError> = get_text_retry(req_uri).await;
    let result = match res {
        Ok(text) => {
            let object: Result<ESICorporation, serde_json::Error> = serde_json::from_str(&text);
            match object {
                Ok(object) => {
                    let insertable =
                        crate::entity::corporations::ActiveModel::from_esi(corporation_id, object);
                    Ok(insertable)
                }
                Err(e) => {
                    error!(
                        "Couldn't deserialize corporation {}: {:?}",
                        corporation_id, e
                    );
                    Err(EsiError::from(e))
                }
            }
        }
        Err(e) => {
            error!("Couldn't fetch corporation {}: {:?}", corporation_id, e);
            Err(e)
        }
    };
    result
}

async fn get_text(url: &str) -> Result<String, EsiError> {
    info!("Sending request to {}", url);
    let result = reqwest::get(url).await?;
    match result.error_for_status() {
        Ok(response) => {
            let body = response.text().await?;
            Ok(body)
        }
        Err(e) => Err(EsiError::ApiError(e)),
    }
}

pub async fn get_killmail(req: &ESIKillmailRequest) -> Result<ESIKillmail, EsiError> {
    let request_uri = get_uri(format!("killmails/{}/{}", req.id, req.hash));
    let resp = get_text_retry(request_uri).await?;
    let object: ESIKillmail = serde_json::from_str(&resp)?;
    Ok(object)
}

async fn get_text_retry(url: String) -> Result<String, EsiError> {
    let mut retry_attempts = RETRY_COUNT;
    let result: Result<String, EsiError> = loop {
        match get_text(&url).await {
            Ok(result) => {
                break Ok(result);
            }
            Err(e) => {
                if retry_attempts > 0 {
                    retry_attempts -= 1;
                    let retry_delay = (11 - retry_attempts) * 100;
                    warn!(
                        "Got error {:?} while fetching {}, retrying in at least {}ms ({} attempts remain)",
                        e, &url, retry_delay, retry_attempts
                    );
                    sleep(Duration::from_millis(retry_delay)).await;
                    continue;
                } else {
                    break Err(e);
                }
            }
        }
    };
    result
}

pub async fn get_factions() -> Result<Vec<ESIFaction>, EsiError> {
    let request_uri = get_uri("universe/factions".to_string());
    let text_result = get_text_retry(request_uri).await?;
    let results: Vec<ESIFaction> = serde_json::from_str(&text_result)?;
    Ok(results)
}
