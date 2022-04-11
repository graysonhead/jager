use chrono::NaiveDateTime;
use serde::de;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct ESICategory {
    pub category_id: u64,
    pub groups: Vec<u64>,
    pub name: String,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIKillmail {
    pub killmail_id: u64,
    #[serde(deserialize_with = "from_zulu_timestamp")]
    pub killmail_time: NaiveDateTime,
    pub solar_system_id: u64,
    pub victim: ESIVictim,
    pub attackers: Vec<ESIAttacker>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIKillPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIFaction {
    pub faction_id: u64,
    pub corporation_id: Option<u64>,
    pub militia_corporation_id: Option<u64>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESIAlliance {
    pub faction_id: Option<u64>,
    pub name: String,
    pub ticker: String,
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
