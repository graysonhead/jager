//! SeaORM Entity. Generated by sea-orm-codegen 0.2.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "alliances")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub alliance_id: u64,
    pub faction_id: Option<u64>,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub ticker: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::factions::Entity",
        from = "Column::FactionId",
        to = "super::factions::Column::FactionId",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Factions,
    #[sea_orm(has_many = "super::attackers::Entity")]
    Attackers,
    #[sea_orm(has_many = "super::corporations::Entity")]
    Corporations,
    #[sea_orm(has_many = "super::victims::Entity")]
    Victims,
}

impl Related<super::factions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Factions.def()
    }
}

impl Related<super::attackers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attackers.def()
    }
}

impl Related<super::corporations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Corporations.def()
    }
}

impl Related<super::victims::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Victims.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}