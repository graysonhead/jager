use crate::database;
use crate::database::JagerDatabaseError;
use crate::entity::prelude::*;
use crate::entity::*;
use crate::esi;
use crate::esi::EsiError;
use crate::organization_processing;
use database::CreateOrExist;
use datamodels::esi_models::{
    ESIAttacker, ESIKillPosition, ESIKillmail, ESIKillmailRequest, ESIVictim,
};
use futures::{stream, StreamExt};
use pbr::ProgressBar;
use sea_orm::prelude::*;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;

#[derive(Debug)]
pub enum ProcessingError {
    ESIError(EsiError),
    DBError(DbErr),
    JagerDatabaseError(JagerDatabaseError),
}

impl From<EsiError> for ProcessingError {
    fn from(err: EsiError) -> ProcessingError {
        ProcessingError::ESIError(err)
    }
}

impl From<DbErr> for ProcessingError {
    fn from(err: DbErr) -> ProcessingError {
        ProcessingError::DBError(err)
    }
}

impl From<JagerDatabaseError> for ProcessingError {
    fn from(err: JagerDatabaseError) -> ProcessingError {
        ProcessingError::JagerDatabaseError(err)
    }
}

pub async fn process_victim(
    db: &DatabaseConnection,
    victim: ESIVictim,
    killmail_id: u64,
) -> Result<(), ProcessingError> {
    if let Some(alliance_id) = victim.alliance_id {
        organization_processing::store_alliance_if_not_present(db, alliance_id).await?;
    }
    if let Some(corp_id) = victim.corporation_id {
        organization_processing::store_corporation_if_not_present(db, corp_id).await?;
    }
    if let Some(char_id) = victim.character_id {
        organization_processing::store_pubchar_info_if_not_present(db, char_id).await?;
    }
    let victim_insertable = victims::ActiveModel::from_esi(victim, killmail_id);
    database::insert_single(db, victim_insertable).await?;
    Ok(())
}

pub async fn process_attackers(
    db: &DatabaseConnection,
    attackers: Vec<ESIAttacker>,
    killmail_id: u64,
) -> Result<(), ProcessingError> {
    // Create characters
    let character_ids: Vec<u64> = attackers
        .clone()
        .into_iter()
        .filter_map(|option| option.character_id)
        .collect();
    let corporation_ids: Vec<u64> = attackers
        .clone()
        .into_iter()
        .filter_map(|character| character.corporation_id)
        .collect();
    let alliance_ids: Vec<u64> = attackers
        .clone()
        .into_iter()
        .filter_map(|character| character.alliance_id)
        .collect();
    organization_processing::store_alliances_if_not_present(db, alliance_ids).await?;
    organization_processing::store_corporations_if_not_present(db, corporation_ids).await?;
    organization_processing::store_pubchars_info_if_not_present(db, character_ids).await?;
    let attacker_insertables: Vec<attackers::ActiveModel> = attackers
        .into_iter()
        .map(|attacker| attackers::ActiveModel::from_esi(attacker, killmail_id))
        .collect();
    database::insert_multiple_attackers(db, attacker_insertables).await?;
    Ok(())
}

pub async fn process_position(
    db: &DatabaseConnection,
    position: ESIKillPosition,
    killmail_id: u64,
) -> Result<(), ProcessingError> {
    let position_insertable: killmail_positions::ActiveModel =
        killmail_positions::ActiveModel::from_esi(position, killmail_id);
    database::insert_single(db, position_insertable).await?;
    Ok(())
}

pub async fn process_killmail(
    db: &DatabaseConnection,
    killmail: ESIKillmail,
) -> Result<(), ProcessingError> {
    let killmail_id = killmail.killmail_id;
    let killmail_attackers = killmail.clone().attackers;
    let killmail_victim = killmail.clone().victim;
    let killmail_insertable = killmails::ActiveModel::from(killmail);
    let killmail_position = killmail_victim.clone().position;
    // If killmail doesn't exist, create it. If it does exist, exit
    match database::insert_killmail_if_not_present(db, killmail_insertable).await? {
        CreateOrExist::Exists => {
            info!(
                "Killmail {} exists in db already, skipping processing",
                killmail_id
            );
            Ok(())
        }
        CreateOrExist::Created => {
            // fetch victim pubchar info and insert victim
            process_victim(db, killmail_victim, killmail_id).await?;
            // fetch attackers pubchar info and insert attackers
            process_attackers(db, killmail_attackers, killmail_id).await?;
            // if there is a position, insert the position
            if let Some(position) = killmail_position {
                process_position(db, position, killmail_id).await?;
            }
            Ok(())
        }
    }
}

pub async fn process_esi_killmail(
    db: &DatabaseConnection,
    request: ESIKillmailRequest,
) -> Result<(), ProcessingError> {
    // get killmail from esi
    // TODO: Check if exists here
    if Killmails::find()
        .filter(killmails::Column::KillmailId.eq(request.id.clone()))
        .one(db)
        .await?
        .is_none()
    {
        let killmail = esi::get_killmail(&request).await?;
        process_killmail(db, killmail).await?;
        Ok(())
    } else {
        Ok(())
    }
}

pub async fn process_killmails(
    db: &DatabaseConnection,
    requests: Vec<ESIKillmailRequest>,
    pb: &mut ProgressBar<std::io::Stdout>,
) -> Result<(), ProcessingError> {
    let mut bodies = stream::iter(requests)
        .map(|req| async move { process_esi_killmail(db, req).await })
        .buffer_unordered(20);
    while let Some(result) = bodies.next().await {
        match result {
            Ok(_) => {
                info!("Processed killmail");
                pb.inc();
            }
            Err(e) => {
                error!("Couldn't process killmail: {:?}", e);
                pb.inc();
            }
        }
    }
    Ok(())
}
