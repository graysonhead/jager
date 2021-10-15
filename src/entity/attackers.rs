//! SeaORM Entity. Generated by sea-orm-codegen 0.2.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "attackers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub attacker_id: u64,
    pub character_id: Option<u64>,
    pub alliance_id: Option<u64>,
    pub corporation_id: Option<u64>,
    pub faction_id: Option<u64>,
    pub damage_done: u64,
    pub final_blow: bool,
    pub security_status: f32,
    pub ship_type_id: Option<u64>,
    pub weapon_type_id: Option<u64>,
    pub killmail_id: u64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::killmails::Entity",
        from = "Column::KillmailId",
        to = "super::killmails::Column::KillmailId",
        on_update = "Restrict",
        on_delete = "Cascade"
    )]
    Killmails,
    #[sea_orm(
        belongs_to = "super::character_public_info::Entity",
        from = "Column::CharacterId",
        to = "super::character_public_info::Column::CharacterId",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    CharacterPublicInfo,
    #[sea_orm(
        belongs_to = "super::esi_types::Entity",
        from = "Column::ShipTypeId",
        to = "super::esi_types::Column::TypeId",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    EsiTypes,
    #[sea_orm(
        belongs_to = "super::alliances::Entity",
        from = "Column::AllianceId",
        to = "super::alliances::Column::AllianceId",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Alliances,
    #[sea_orm(
        belongs_to = "super::corporations::Entity",
        from = "Column::CorporationId",
        to = "super::corporations::Column::CorporationId",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Corporations,
    #[sea_orm(
        belongs_to = "super::factions::Entity",
        from = "Column::FactionId",
        to = "super::factions::Column::FactionId",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Factions,
}

impl Related<super::killmails::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Killmails.def()
    }
}

impl Related<super::character_public_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CharacterPublicInfo.def()
    }
}

impl Related<super::esi_types::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EsiTypes.def()
    }
}

impl Related<super::alliances::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Alliances.def()
    }
}

impl Related<super::corporations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Corporations.def()
    }
}

impl Related<super::factions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Factions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
