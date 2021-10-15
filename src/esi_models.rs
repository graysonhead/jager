use chrono::NaiveDateTime;
use sea_orm::{Set, Unset};
use serde::de;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct ESICategory {
    pub category_id: u64,
    pub groups: Vec<u64>,
    pub name: String,
}

impl From<ESICategory> for crate::entity::esi_categories::ActiveModel {
    fn from(item: ESICategory) -> Self {
        crate::entity::esi_categories::ActiveModel {
            category_id: Set(item.category_id),
            category_name: Set(item.name),
        }
    }
}

impl From<ESIGroup> for crate::entity::esi_groups::ActiveModel {
    fn from(item: ESIGroup) -> Self {
        crate::entity::esi_groups::ActiveModel {
            group_id: Set(item.group_id),
            group_name: Set(item.name),
            category_id: Set(item.category_id),
        }
    }
}

impl From<ESIType> for crate::entity::esi_types::ActiveModel {
    fn from(item: ESIType) -> Self {
        crate::entity::esi_types::ActiveModel {
            type_id: Set(item.type_id),
            description: Set(item.description),
            group_id: Set(item.group_id),
            type_name: Set(item.name),
            mass: Set(item.mass),
        }
    }
}

pub struct NaiveDateTimeVisitor;

impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "A string that represents chrono::NaiveDateTime, but will ignore timezones"
        )
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
            Ok(t) => Ok(t),
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
        }
    }
}

pub fn from_zulu_timestamp<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(NaiveDateTimeVisitor)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ESIGroup {
    pub category_id: u64,
    pub group_id: u64,
    pub name: String,
    pub types: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ESIType {
    pub type_id: u64,
    pub description: String,
    pub group_id: u64,
    pub name: String,
    pub mass: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ESIKillmailRequest {
    pub id: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EsiCharacterPublicInfo {
    pub alliance_id: Option<u64>,
    pub birthday: Option<String>,
    pub corporation_id: u64,
    pub faction_id: Option<u64>,
    pub name: String,
    pub security_status: f32,
}

impl crate::entity::character_public_info::ActiveModel {
    pub fn from_esi(
        char_id: u64,
        char_public_info: &EsiCharacterPublicInfo,
    ) -> crate::entity::character_public_info::ActiveModel {
        let birthday: Option<String> = match &char_public_info.birthday {
            Some(bd) => Some(bd.to_string()),
            None => None,
        };
        crate::entity::character_public_info::ActiveModel {
            character_id: Set(char_id),
            alliance_id: Set(char_public_info.alliance_id),
            character_name: Set(char_public_info.name.to_string()),
            birthday: Set(birthday),
            corporation_id: Set(char_public_info.corporation_id),
            faction_id: Set(char_public_info.faction_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIKillmail {
    pub killmail_id: u64,
    #[serde(deserialize_with = "from_zulu_timestamp")]
    pub killmail_time: NaiveDateTime,
    pub solar_system_id: u64,
    pub victim: ESIVictim,
    pub attackers: Vec<ESIAttacker>,
}

impl From<ESIKillmail> for crate::entity::killmails::ActiveModel {
    fn from(item: ESIKillmail) -> Self {
        crate::entity::killmails::ActiveModel {
            killmail_id: Set(item.killmail_id),
            killmail_time: Set(item.killmail_time),
            solar_system_id: Set(item.solar_system_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIVictim {
    pub alliance_id: Option<u64>,
    pub character_id: Option<u64>,
    pub faction_id: Option<u64>,
    pub corporation_id: Option<u64>,
    pub damage_taken: u64,
    pub ship_type_id: u64,
    pub position: Option<ESIKillPosition>,
}

impl crate::entity::victims::ActiveModel {
    pub fn from_esi(item: ESIVictim, km_id: u64) -> Self {
        Self {
            victim_id: Unset(None),
            alliance_id: Set(item.alliance_id),
            character_id: Set(item.character_id),
            faction_id: Set(item.faction_id),
            corporation_id: Set(item.corporation_id),
            damage_taken: Set(item.damage_taken),
            ship_type_id: Set(item.ship_type_id),
            killmail_id: Set(km_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIAttacker {
    pub alliance_id: Option<u64>,
    pub character_id: Option<u64>,
    pub corporation_id: Option<u64>,
    pub faction_id: Option<u64>,
    pub damage_done: u64,
    pub final_blow: bool,
    pub security_status: f32,
    pub ship_type_id: Option<u64>,
    pub weapon_type_id: Option<u64>,
}

impl crate::entity::attackers::ActiveModel {
    pub fn from_esi(item: ESIAttacker, km_id: u64) -> Self {
        Self {
            attacker_id: Unset(None),
            character_id: Set(item.character_id),
            alliance_id: Set(item.alliance_id),
            corporation_id: Set(item.corporation_id),
            faction_id: Set(item.faction_id),
            damage_done: Set(item.damage_done),
            final_blow: Set(item.final_blow),
            security_status: Set(item.security_status),
            ship_type_id: Set(item.ship_type_id),
            weapon_type_id: Set(item.weapon_type_id),
            killmail_id: Set(km_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIKillPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl crate::entity::killmail_positions::ActiveModel {
    pub fn from_esi(item: ESIKillPosition, killmail_id: u64) -> Self {
        Self {
            position_id: Unset(None),
            x: Set(item.x),
            y: Set(item.y),
            z: Set(item.z),
            killmail_id: Set(killmail_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIFaction {
    pub faction_id: u64,
    pub corporation_id: Option<u64>,
    pub militia_corporation_id: Option<u64>,
    pub name: String,
}

impl From<ESIFaction> for crate::entity::factions::ActiveModel {
    fn from(item: ESIFaction) -> Self {
        Self {
            faction_id: Set(item.faction_id),
            corporation_id: Set(item.corporation_id),
            militia_corporation_id: Set(item.militia_corporation_id),
            name: Set(item.name),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIAlliance {
    pub faction_id: Option<u64>,
    pub name: String,
    pub ticker: String,
}

impl crate::entity::alliances::ActiveModel {
    pub fn from_esi(alliance_id: u64, item: ESIAlliance) -> Self {
        Self {
            alliance_id: Set(alliance_id),
            faction_id: Set(item.faction_id),
            name: Set(item.name),
            ticker: Set(item.ticker),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESICorporation {
    pub alliance_id: Option<u64>,
    pub faction_id: Option<u64>,
    pub member_count: Option<u64>,
    pub name: String,
    pub ticker: String,
    pub war_eligible: Option<bool>,
}

impl crate::entity::corporations::ActiveModel {
    pub fn from_esi(corporation_id: u64, item: ESICorporation) -> Self {
        Self {
            corporation_id: Set(corporation_id),
            alliance_id: Set(item.alliance_id),
            faction_id: Set(item.faction_id),
            member_count: Set(item.member_count),
            name: Set(item.name),
            ticker: Set(item.ticker),
            war_eligible: Set(item.war_eligible),
        }
    }
}
