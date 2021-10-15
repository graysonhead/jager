use dotenv::dotenv;
use futures::{stream, StreamExt};
use sea_orm::{
    ActiveModelTrait, Database, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
};
use std::env;
use tokio::time::{sleep, Duration};

/// Get a database connection from environment variable
pub async fn establish_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");
    Ok(Database::connect(&database_url).await?)
}

pub enum CreateOrExist {
    Created,
    Exists,
}

pub async fn insert_single<T: ActiveModelTrait>(
    db: &DatabaseConnection,
    item: T,
) -> Result<T, DbErr>
where
    <<T as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Model: IntoActiveModel<T>,
    T: std::marker::Send,
{
    item.insert(db).await
}

pub async fn insert_retry<T: ActiveModelTrait>(db: &DatabaseConnection, item: T) -> Result<T, DbErr>
where
    <<T as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Model: IntoActiveModel<T>,
    T: std::marker::Send,
{
    let mut retry_attempts = 10;
    let result = loop {
        match insert_single(db, item.clone()).await {
            Ok(result) => break Ok(result),
            Err(e) => {
                if e.to_string().contains("Duplicate entry") {
                    break Err(e);
                }
                if retry_attempts > 0 {
                    retry_attempts -= 1;
                    let retry_delay = (11 - retry_attempts) * 100;
                    warn!(
                        "Got error {:?} while inserting, retrying in at least {}ms ({} attempts remain)",
                        e, retry_delay, retry_attempts
                    );
                    sleep(Duration::from_millis(retry_delay)).await;
                    continue;
                } else {
                    break Err(e);
                }
            }
        }
    };
    result
}

pub async fn insert_category_if_not_present(
    db: &DatabaseConnection,
    category: crate::entity::esi_categories::ActiveModel,
) -> Result<(), DbErr> {
    let category_id: u64 = category.category_id.clone().into_value().unwrap();
    let result = insert_retry(db, category).await;
    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            if err.to_string().contains("Duplicate entry") {
                info!("Category {} already exists:", category_id);
                Ok(())
            } else {
                error!("Couldn't insert type {}: {:?}", category_id, err);
                Err(err)
            }
        }
    }
}

pub async fn insert_group_if_not_present(
    db: &DatabaseConnection,
    group: crate::entity::esi_groups::ActiveModel,
) -> Result<(), DbErr> {
    let group_id: u64 = group.group_id.clone().into_value().unwrap();
    let result = insert_retry(db, group).await;
    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            if err.to_string().contains("Duplicate entry") {
                info!("Group {} already exists:", group_id);
                Ok(())
            } else {
                error!("Couldn't insert type {}: {:?}", group_id, err);
                Err(err)
            }
        }
    }
}

pub async fn insert_type_if_not_present(
    db: &DatabaseConnection,
    esi_type: crate::entity::esi_types::ActiveModel,
) -> Result<(), DbErr> {
    let type_id: u64 = esi_type.type_id.clone().into_value().unwrap();
    let result = insert_retry(db, esi_type).await;
    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            if err.to_string().contains("Duplicate entry") {
                info!("Type {} already exists", type_id);
                Ok(())
            } else {
                error!("Couldn't insert type {}: {:?}", type_id, err);
                Err(err)
            }
        }
    }
}

pub async fn insert_pubchar_info_if_not_present(
    db: &DatabaseConnection,
    public_info: crate::entity::character_public_info::ActiveModel,
) -> Result<CreateOrExist, DbErr> {
    let character_id: u64 = public_info.character_id.clone().into_value().unwrap();
    match insert_retry(db, public_info).await {
        Ok(_) => Ok(CreateOrExist::Created),
        Err(err) => {
            if err.to_string().contains("Duplicate entry") {
                Ok(CreateOrExist::Exists)
            } else {
                error!("Couldn't insert pubchar_info {}: {:?}", character_id, err);
                Err(err)
            }
        }
    }
}

pub async fn insert_alliance_if_not_present(
    db: &DatabaseConnection,
    alliance: crate::entity::alliances::ActiveModel,
) -> Result<CreateOrExist, DbErr> {
    let alliance_id: u64 = alliance.alliance_id.clone().into_value().unwrap();
    match insert_retry(db, alliance).await {
        Ok(_) => Ok(CreateOrExist::Created),
        Err(err) => {
            if is_duplicate_err(&err) {
                Ok(CreateOrExist::Exists)
            } else {
                error!("Couldn't insert alliance {}: {:?}", alliance_id, err);
                Err(err)
            }
        }
    }
}

pub async fn insert_corporation_if_not_present(
    db: &DatabaseConnection,
    corporation: crate::entity::corporations::ActiveModel,
) -> Result<CreateOrExist, DbErr> {
    let corporation_id: u64 = corporation.corporation_id.clone().into_value().unwrap();
    match insert_retry(db, corporation).await {
        Ok(_) => Ok(CreateOrExist::Created),
        Err(err) => {
            if is_duplicate_err(&err) {
                Ok(CreateOrExist::Exists)
            } else {
                error!("Couldn't insert corporation {}: {:?}", corporation_id, err);
                Err(err)
            }
        }
    }
}

pub async fn insert_corporations_if_not_present(
    db: &DatabaseConnection,
    corporations: Vec<crate::entity::corporations::ActiveModel>,
) -> Result<(), DbErr> {
    let mut error: Option<DbErr> = Option::None;
    let mut bodies = stream::iter(corporations)
        .map(|corp| async move { insert_corporation_if_not_present(db, corp).await })
        .buffer_unordered(10);
    while let Some(result) = bodies.next().await {
        match result {
            Ok(_) => info!("Inserted corporation"),
            Err(e) => error = Some(e),
        }
    }
    if let Some(err) = error {
        Err(err)
    } else {
        Ok(())
    }
}

pub async fn insert_pubchars_if_not_present(
    db: &DatabaseConnection,
    public_infos: Vec<crate::entity::character_public_info::ActiveModel>,
) -> Result<(), DbErr> {
    let mut error: Option<DbErr> = Option::None;
    let mut bodies = stream::iter(public_infos)
        .map(|pubinfo| async move { insert_pubchar_info_if_not_present(db, pubinfo).await })
        .buffer_unordered(10);
    while let Some(result) = bodies.next().await {
        match result {
            Ok(_) => info!("Inserted attacker"),
            Err(e) => error = Some(e),
        }
    }
    if let Some(err) = error {
        Err(err)
    } else {
        Ok(())
    }
}

pub async fn insert_alliances_if_not_present(
    db: &DatabaseConnection,
    alliances: Vec<crate::entity::alliances::ActiveModel>,
) -> Result<(), DbErr> {
    let mut error: Option<DbErr> = Option::None;
    let mut bodies = stream::iter(alliances)
        .map(|alliance| async move { insert_alliance_if_not_present(db, alliance).await })
        .buffer_unordered(10);
    while let Some(result) = bodies.next().await {
        match result {
            Ok(_) => info!("Inserted alliance"),
            Err(e) => error = Some(e),
        }
    }
    if let Some(err) = error {
        Err(err)
    } else {
        Ok(())
    }
}

pub async fn insert_killmail_if_not_present(
    db: &DatabaseConnection,
    killmail: crate::entity::killmails::ActiveModel,
) -> Result<CreateOrExist, DbErr> {
    let killmail_id: u64 = killmail.killmail_id.clone().into_value().unwrap();
    match insert_retry(db, killmail).await {
        Ok(_) => Ok(CreateOrExist::Created),
        Err(err) => {
            if is_duplicate_err(&err) {
                Ok(CreateOrExist::Exists)
            } else {
                error!("Couldn't insert killmail {}: {:?}", killmail_id, err);
                Err(err)
            }
        }
    }
}

pub fn is_duplicate_err(err: &DbErr) -> bool {
    err.to_string().contains("Duplicate entry")
}

pub async fn insert_faction_if_not_present(
    db: &DatabaseConnection,
    faction: crate::entity::factions::ActiveModel,
) -> Result<CreateOrExist, DbErr> {
    let faction_id: u64 = faction.faction_id.clone().into_value().unwrap();
    match insert_retry(db, faction).await {
        Ok(_) => Ok(CreateOrExist::Created),
        Err(err) => {
            if is_duplicate_err(&err) {
                Ok(CreateOrExist::Exists)
            } else {
                error!("Couldn't insert faction {}: {:?}", faction_id, err);
                Err(err)
            }
        }
    }
}

pub async fn insert_if_not_present_types(
    db: &DatabaseConnection,
    esi_types: Vec<crate::entity::esi_types::ActiveModel>,
) {
    let mut inserts = stream::iter(esi_types)
        .map(|item| insert_type_if_not_present(db, item))
        .buffer_unordered(10);
    while let Some(res) = inserts.next().await {
        match res {
            Ok(result) => info!("Processed: {:?}", result),
            Err(e) => error!("Got error: {:?}", e),
        }
    }
}

pub async fn insert_if_not_present_groups(
    db: &DatabaseConnection,
    groups: Vec<crate::entity::esi_groups::ActiveModel>,
) {
    let mut inserts = stream::iter(groups)
        .map(|item| insert_group_if_not_present(db, item))
        .buffer_unordered(10);
    while let Some(res) = inserts.next().await {
        match res {
            Ok(result) => info!("Processed {:?}", result),
            Err(e) => error!("Got error {:?} while storing group", e),
        }
    }
}

pub async fn insert_if_not_present_factions(
    db: &DatabaseConnection,
    factions: Vec<crate::entity::factions::ActiveModel>,
) {
    let mut inserts = stream::iter(factions)
        .map(|item| insert_faction_if_not_present(db, item))
        .buffer_unordered(10);
    while let Some(res) = inserts.next().await {
        match res {
            Ok(_) => {}
            Err(e) => error!("Got error {:?} while storing faction", e),
        }
    }
}

pub async fn insert_if_not_present_categories(
    db: &DatabaseConnection,
    categories: Vec<crate::entity::esi_categories::ActiveModel>,
) {
    let mut inserts = stream::iter(categories)
        .map(|item| insert_category_if_not_present(db, item))
        .buffer_unordered(10);
    while let Some(res) = inserts.next().await {
        match res {
            Ok(result) => info!("Processed {:?}", result),
            Err(e) => error!("Got error {:?}", e),
        }
    }
}

pub async fn insert_multiple_attackers(
    db: &DatabaseConnection,
    attackers: Vec<crate::entity::attackers::ActiveModel>,
) -> Result<(), DbErr> {
    crate::entity::attackers::Entity::insert_many(attackers)
        .exec(db)
        .await?;
    Ok(())
}
