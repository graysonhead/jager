use backend::database;
use backend::entity::{esi_categories, esi_groups, esi_types, factions};
use backend::esi;

#[tokio::main]
async fn main() {
    backend::logging::setup_logging();
    let db = database::establish_connection().await.unwrap();
    let esi_factions_list = esi::get_factions().await.unwrap();
    let faction_insertables = esi_factions_list
        .into_iter()
        .map(factions::ActiveModel::from)
        .collect();
    database::insert_if_not_present_factions(&db, faction_insertables).await;
    let esi_category_list = esi::get_category_list().await.unwrap();
    let esi_group_list = esi::get_group_list().await.unwrap();
    let esi_type_list = esi::get_type_list().await.unwrap();
    let esi_categories = esi::get_esi_categories(esi_category_list).await;
    let category_insertables: Vec<esi_categories::ActiveModel> = esi_categories
        .into_iter()
        .map(esi_categories::ActiveModel::from)
        .collect();
    database::insert_if_not_present_categories(&db, category_insertables).await;
    let esi_groups = esi::get_esi_groups(esi_group_list).await;
    let group_insertables: Vec<esi_groups::ActiveModel> = esi_groups
        .into_iter()
        .map(esi_groups::ActiveModel::from)
        .collect();
    database::insert_if_not_present_groups(&db, group_insertables).await;
    let esi_types = esi::get_esi_types(esi_type_list).await;
    let type_insertables: Vec<esi_types::ActiveModel> = esi_types
        .into_iter()
        .map(esi_types::ActiveModel::from)
        .collect();
    database::insert_if_not_present_types(&db, type_insertables).await;
}
