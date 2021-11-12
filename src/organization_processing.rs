use crate::database;
use crate::entity::prelude::*;
use crate::entity::*;
use crate::esi;
use crate::killmail_processing::ProcessingError;
use futures::{stream, StreamExt};
use sea_orm::prelude::*;
use sea_orm::DatabaseConnection;

const CONCURRENCY: usize = 10;

pub async fn store_alliance_if_not_present(
    db: &DatabaseConnection,
    alliance_id: u64,
) -> Result<(), ProcessingError> {
    let alliance_result = Alliances::find()
        .filter(alliances::Column::AllianceId.eq(alliance_id))
        .one(db)
        .await?;
    if alliance_result.is_none() {
        info!("Alliance {} not in DB, fetching from esi", alliance_id);
        let alliance_insertable = esi::get_alliance(alliance_id).await?;
        database::insert_alliance_if_not_present(db, alliance_insertable).await?;
    }
    Ok(())
}

pub async fn store_alliances_if_not_present(
    db: &DatabaseConnection,
    alliance_ids: Vec<u64>,
) -> Result<(), ProcessingError> {
    let mut err: Option<ProcessingError> = Option::None;
    let mut bodies = stream::iter(alliance_ids)
        .map(|id| async move { store_alliance_if_not_present(db, id).await })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(_) => {}
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(())
    }
}

pub async fn store_corporation_if_not_present(
    db: &DatabaseConnection,
    corporation_id: u64,
) -> Result<(), ProcessingError> {
    let corporation_result = Corporations::find()
        .filter(corporations::Column::CorporationId.eq(corporation_id))
        .one(db)
        .await?;
    if corporation_result.is_none() {
        info!(
            "Corporation {} not in db, fetching from esi",
            corporation_id
        );
        let corporation_insertable = esi::get_corporation(corporation_id).await?;
        database::insert_corporation_if_not_present(db, corporation_insertable).await?;
    }
    Ok(())
}

pub async fn store_corporations_if_not_present(
    db: &DatabaseConnection,
    corporation_ids: Vec<u64>,
) -> Result<(), ProcessingError> {
    let mut err: Option<ProcessingError> = Option::None;
    let mut bodies = stream::iter(corporation_ids)
        .map(|id| async move { store_corporation_if_not_present(db, id).await })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(_) => {}
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(())
    }
}

pub async fn store_pubchar_info_if_not_present(
    db: &DatabaseConnection,
    char_id: u64,
) -> Result<(), ProcessingError> {
    let pubchar_result = CharacterPublicInfo::find()
        .filter(character_public_info::Column::CharacterId.eq(char_id))
        .one(db)
        .await?;
    if pubchar_result.is_none() {
        info!("Pubchar info for {} not found, fetching from esi", char_id);
        let pubchar_insertable = esi::get_character(char_id).await?;
        database::insert_pubchar_info_if_not_present(db, pubchar_insertable).await?;
    }
    Ok(())
}

pub async fn store_pubchars_info_if_not_present(
    db: &DatabaseConnection,
    pubchar_ids: Vec<u64>,
) -> Result<(), ProcessingError> {
    let mut err: Option<ProcessingError> = Option::None;
    let mut bodies = stream::iter(pubchar_ids)
        .map(|id| async move { store_pubchar_info_if_not_present(db, id).await })
        .buffer_unordered(CONCURRENCY);
    while let Some(item) = bodies.next().await {
        match item {
            Ok(_) => {}
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(())
    }
}
