//! SeaORM Entity. Generated by sea-orm-codegen 0.2.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "killmail_positions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub position_id: u64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
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
}

impl Related<super::killmails::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Killmails.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}