use crate::entity::prelude::*;
use crate::entity::*;
use druid::Data;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize, Clone, Data)]
pub struct KillLossRatio {
    pub kills: usize,
    pub losses: usize,
}
#[derive(Debug, Serialize, Deserialize, Clone, Data)]
pub struct CharacterStats {
    pub kill_loss_ratio: KillLossRatio,
}

fn get_kill_loss_ratio(kills: &[attackers::Model], losses: &[victims::Model]) -> KillLossRatio {
    KillLossRatio {
        kills: kills.len(),
        losses: losses.len(),
    }
}

pub async fn get_character_stats(
    db: DatabaseConnection,
    name: String,
) -> Result<Option<CharacterStats>, DbErr> {
    let start_time = Instant::now();
    let character_info_result = CharacterPublicInfo::find()
        .filter(character_public_info::Column::CharacterName.eq(name))
        .one(&db)
        .await?;
    match character_info_result {
        Some(char_info) => {
            let kills = Attackers::find()
                .filter(attackers::Column::CharacterId.eq(char_info.character_id))
                .all(&db)
                .await?;
            let losses = Victims::find()
                .filter(victims::Column::CharacterId.eq(char_info.character_id))
                .all(&db)
                .await?;
            let kill_loss_ratio = get_kill_loss_ratio(&kills, &losses);
            let end_time = Instant::now();
            let duration = (end_time - start_time).as_millis();
            info!("Request took {}ms", duration);
            Ok(Some(CharacterStats { kill_loss_ratio }))
        }
        None => Ok(None),
    }
}

// pub async fn get_character_stats_eager(db: DatabaseConnection, name: String) {
//     let start_time = Instant::now();
//     let char:  = CharacterPublicInfo::find()
//         .find_with_related
// }
