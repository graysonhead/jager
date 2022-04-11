use crate::entity::prelude::*;
use crate::entity::*;
use crate::esi;
use crate::killmail_processing::ProcessingError;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::Set;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KillLossRatio {
    pub kills: usize,
    pub losses: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharInfo {
    pub alliance_name: Option<String>,
    pub alliance_ticker: Option<String>,
    pub corporation_name: Option<String>,
    pub corporation_ticker: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterStats {
    pub char_info: CharInfo,
    pub kill_loss_ratio: KillLossRatio,
    pub solo_kill_loss_ratio: KillLossRatio,
}

fn get_kill_loss_ratio(kills: &[StatsKillmail], losses: &[StatsKillmail]) -> KillLossRatio {
    KillLossRatio {
        kills: kills.len(),
        losses: losses.len(),
    }
}

fn get_attacker_player_count(attackers: &[attackers::Model]) -> usize {
    let attackers_clone: Vec<attackers::Model> = attackers.to_vec();
    attackers_clone
        .into_iter()
        .filter(|attacker| attacker.character_id.is_some())
        .count()
}

fn get_solo_kill_loss_ratio(kills: &[StatsKillmail], losses: &[StatsKillmail]) -> KillLossRatio {
    let solo_kills = kills
        .iter()
        .filter(|kill| get_attacker_player_count(&kill.attackers) == 1)
        .count();
    let solo_losses = losses
        .iter()
        .filter(|loss| get_attacker_player_count(&loss.attackers) == 1)
        .count();
    KillLossRatio {
        kills: solo_kills,
        losses: solo_losses,
    }
}

struct StatsKillmail {
    killmail_id: u64,
    killmail_time: NaiveDateTime,
    solar_system_id: u64,
    victim: victims::Model,
    attackers: Vec<attackers::Model>,
    position: killmail_positions::Model,
}

async fn get_statskillmail_from_id(
    db: &DatabaseConnection,
    killmail_id: u64,
) -> Result<Option<StatsKillmail>, DbErr> {
    let attackers = Attackers::find()
        .filter(attackers::Column::KillmailId.eq(killmail_id))
        .all(db)
        .await?;
    let victim = Victims::find()
        .filter(victims::Column::KillmailId.eq(killmail_id))
        .one(db)
        .await?;
    if let Some((killmail, position)) = Killmails::find()
        .filter(killmails::Column::KillmailId.eq(killmail_id))
        .find_with_related(KillmailPositions)
        .one(db)
        .await?
    {
        Ok(Some(StatsKillmail {
            killmail_id: killmail.killmail_id,
            killmail_time: killmail.killmail_time,
            solar_system_id: killmail.solar_system_id,
            victim: victim.unwrap(),
            attackers,
            position: position.unwrap(),
        }))
    } else {
        Ok(None)
    }
}

async fn get_kills_from_list(
    db: &DatabaseConnection,
    kills: Vec<u64>,
) -> Result<Vec<StatsKillmail>, DbErr> {
    let mut results: Vec<StatsKillmail> = Vec::new();
    let mut err: Option<DbErr> = Option::None;
    for attacker_id in kills {
        let killmail_stats_res = get_statskillmail_from_id(&db, attacker_id).await;
        match killmail_stats_res {
            Ok(opt) => {
                if let Some(skm) = opt {
                    results.push(skm);
                }
            }
            Err(e) => {
                err = Some(e);
            }
        }
    }
    if let Some(e) = err {
        Err(e)
    } else {
        Ok(results)
    }
}

async fn get_character_kills(
    db: &DatabaseConnection,
    char_id: u64,
) -> Result<Vec<StatsKillmail>, DbErr> {
    let attacker_list = Attackers::find()
        .filter(attackers::Column::CharacterId.eq(char_id))
        .all(db)
        .await?;
    let killmail_ids = attacker_list
        .into_iter()
        .map(|attacker| attacker.killmail_id)
        .collect();
    get_kills_from_list(db, killmail_ids).await
}

async fn get_character_losses(
    db: &DatabaseConnection,
    char_id: u64,
) -> Result<Vec<StatsKillmail>, DbErr> {
    let victim_list = Victims::find()
        .filter(victims::Column::CharacterId.eq(char_id))
        .all(db)
        .await?;
    let killmail_ids = victim_list
        .into_iter()
        .map(|victim| victim.killmail_id)
        .collect();
    get_kills_from_list(db, killmail_ids).await
}

async fn get_character_alliance(
    db: &DatabaseConnection,
    character_info: &character_public_info::Model,
) -> Result<Option<alliances::Model>, DbErr> {
    if let Some(alliance_id) = character_info.alliance_id {
        let alliance = Alliances::find()
            .filter(alliances::Column::AllianceId.eq(alliance_id))
            .one(db)
            .await?;
        Ok(alliance)
    } else {
        Ok(None)
    }
}

async fn get_character_corporation(
    db: &DatabaseConnection,
    character_info: &character_public_info::Model,
) -> Result<Option<corporations::Model>, DbErr> {
    let corporation = Corporations::find()
        .filter(corporations::Column::CorporationId.eq(character_info.corporation_id))
        .one(db)
        .await?;
    Ok(corporation)
}

fn get_char_info(
    alliance: &Option<alliances::Model>,
    corporation: &Option<corporations::Model>,
) -> CharInfo {
    let alliance_name: Option<String>;
    let alliance_ticker: Option<String>;
    let corporation_name: Option<String>;
    let corporation_ticker: Option<String>;
    if let Some(alliance) = alliance {
        alliance_name = Some(alliance.name.clone());
        alliance_ticker = Some(alliance.ticker.clone());
    } else {
        alliance_name = None;
        alliance_ticker = None;
    }
    if let Some(corporation) = corporation {
        corporation_name = Some(corporation.name.clone());
        corporation_ticker = Some(corporation.ticker.clone());
    } else {
        corporation_name = None;
        corporation_ticker = None;
    }
    CharInfo {
        alliance_name,
        alliance_ticker,
        corporation_name,
        corporation_ticker,
    }
}

pub async fn update_character_public_info(
    db: &DatabaseConnection,
    character: character_public_info::Model,
) -> Result<Option<character_public_info::Model>, ProcessingError> {
    let char_id = character.character_id.clone();
    // let mut active_char: character_public_info::ActiveModel = character.into();
    let mut new_active_model = esi::get_character(char_id.clone()).await?;
    // let new_active_model = character_public_info::ActiveModel::from_esi(char_id.clone(), new_char_info);
    new_active_model.last_updated = Set(Some(DateTime::naive_utc(&Utc::now())));
    let new_info_result = CharacterPublicInfo::find()
        .filter(character_public_info::Column::CharacterId.eq(char_id))
        .one(db)
        .await?;
    Ok(new_info_result)
}

pub async fn get_or_update_character_public_info(
    db: &DatabaseConnection,
    name: String,
) -> Result<Option<character_public_info::Model>, ProcessingError> {
    let character_info_result = CharacterPublicInfo::find()
        .filter(character_public_info::Column::CharacterName.eq(name.clone()))
        .one(db)
        .await?;
    // If the public info hasn't been updated in the last few days, update it
    if let Some(character) = character_info_result {
        if let Some(last_updated) = character.last_updated {
            let now = Utc::now().naive_utc();
            if now - last_updated > Duration::days(3) {
                let new_info_result = update_character_public_info(db, character).await?;
                info!("Info for character {} out of date, updating", name);
                Ok(new_info_result)
            } else {
                Ok(Some(character))
            }
        } else {
            let new_info_result = update_character_public_info(db, character).await?;
            info!("Updated character public info for {}", name);
            Ok(new_info_result)
        }
    } else {
        Ok(None)
    }
}

pub async fn get_character_stats(
    db: &DatabaseConnection,
    name: String,
) -> Result<Option<CharacterStats>, ProcessingError> {
    let start_time = Instant::now();
    let character_info_result = get_or_update_character_public_info(&db, name).await?;
    match character_info_result {
        Some(char_info) => {
            let alliance = get_character_alliance(&db, &char_info).await?;
            let corporation = get_character_corporation(&db, &char_info).await?;
            let character_info = get_char_info(&alliance, &corporation);
            let kills = get_character_kills(&db, char_info.character_id).await?;
            let losses = get_character_losses(&db, char_info.character_id).await?;
            let kill_loss_ratio = get_kill_loss_ratio(&kills, &losses);
            let solo_kill_loss_ratio = get_solo_kill_loss_ratio(&kills, &losses);
            let end_time = Instant::now();
            let duration = (end_time - start_time).as_millis();
            info!("Request took {}ms", duration);
            Ok(Some(CharacterStats {
                char_info: character_info,
                kill_loss_ratio,
                solo_kill_loss_ratio,
            }))
        }
        None => Ok(None),
    }
}
