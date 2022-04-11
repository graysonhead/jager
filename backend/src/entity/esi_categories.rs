//! SeaORM Entity. Generated by sea-orm-codegen 0.2.3

use datamodels::esi_models::ESICategory;
use sea_orm::entity::prelude::*;
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "esi_categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub category_id: u64,
    #[sea_orm(column_type = "Text")]
    pub category_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::esi_groups::Entity")]
    EsiGroups,
}

impl Related<super::esi_groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EsiGroups.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<ESICategory> for ActiveModel {
    fn from(item: ESICategory) -> Self {
        crate::entity::esi_categories::ActiveModel {
            category_id: Set(item.category_id),
            category_name: Set(item.name),
        }
    }
}
