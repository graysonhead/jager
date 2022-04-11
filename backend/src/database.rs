use async_trait::async_trait;
use dotenv::dotenv;
use futures::{stream, StreamExt};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ConnectOptions, Database,
    DatabaseConnection, DbErr, EntityTrait, IntoActiveModel, Value,
};
use sea_orm_rocket::Database as SODatabase;
use sea_orm_rocket::rocket::figment::Figment;
use std::env;
use std::fmt;
use tokio::time::{sleep, Duration};

#[derive(SODatabase, Debug)]
#[database("sea_orm")]
pub struct Db(SeaOrmPool);

#[derive(Debug, Clone)]
pub struct SeaOrmPool {
    pub conn: sea_orm::DatabaseConnection,
}

#[async_trait]
impl sea_orm_rocket::Pool for SeaOrmPool {
    type Error = sea_orm::DbErr;

    type Connection = sea_orm::DatabaseConnection;

    async fn init(_figment: &Figment) -> Result<Self, Self::Error> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");
        let mut opt = ConnectOptions::new(database_url.to_owned());
        opt.max_connections(100)
            .min_connections(10)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);
        let conn = sea_orm::Database::connect(opt).await?;
        Ok(SeaOrmPool { conn })
    }
    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}

#[derive(Debug, Clone)]
pub struct ValueUnwrapError {
    pub message: String,
}

impl fmt::Display for ValueUnwrapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ValueUnwrapError {
    fn new(msg: String) -> ValueUnwrapError {
        ValueUnwrapError { message: msg }
    }
}

#[derive(Debug)]
pub enum JagerDatabaseError {
    DBError(DbErr),
    ValueUnwrapError(ValueUnwrapError),
}

impl From<DbErr> for JagerDatabaseError {
    fn from(err: DbErr) -> JagerDatabaseError {
        JagerDatabaseError::DBError(err)
    }
}

impl From<ValueUnwrapError> for JagerDatabaseError {
    fn from(err: ValueUnwrapError) -> JagerDatabaseError {
        JagerDatabaseError::ValueUnwrapError(err)
    }
}

/// Get a database connection from environment variable
pub async fn establish_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");
    let mut opt = ConnectOptions::new(database_url.to_owned());
    opt.max_connections(1)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);
    Ok(Database::connect(opt).await?)
}

pub async fn try_establish_connection() -> Result<DatabaseConnection, DbErr> {
    let mut retry_attempts = 10;
    let result = loop {
        match establish_connection().await {
            Ok(result) => break Ok(result),
            Err(e) => {
                if retry_attempts > 0 {
                    retry_attempts -= 1;
                    let retry_delay = (11 - retry_attempts) * 100;
                    warn!(
                        "Error {:?} connecting to DB, retrying in at least {}ms ({} attempts remain)",
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

pub enum CreateOrExist {
    Created,
    Exists,
}

pub async fn insert_single<T: ActiveModelBehavior + sea_orm::ActiveModelTrait>(
    db: &DatabaseConnection,
    item: T,
) -> Result<<<T as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Model, DbErr>
where
    T: std::marker::Send + sea_orm::ActiveModelBehavior + sea_orm::ActiveModelTrait,
    <<T as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Model: IntoActiveModel<T>,
{
    item.insert(db).await
}

pub async fn insert_retry<T: ActiveModelTrait>(
    db: &DatabaseConnection,
    item: T,
) -> Result<<<T as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Model, DbErr>
where
    <<T as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Model: IntoActiveModel<T>,
    T: std::marker::Send + sea_orm::ActiveModelBehavior,
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

fn try_activevalue_to_u64(activevalue: ActiveValue<u64>) -> Result<u64, ValueUnwrapError> {
    if let Some(value) = activevalue.into_value() {
        match value {
            Value::BigUnsigned(return_u64_opt) => {
                if let Some(return_u64) = return_u64_opt {
                    Ok(return_u64)
                } else {
                    Err(ValueUnwrapError::new(
                        "Attempted to unwrap value containing None".to_string(),
                    ))
                }
            }
            _ => Err(ValueUnwrapError::new(
                "Attempted to unwrap non-bigint to u64".to_string(),
            )),
        }
    } else {
        Err(ValueUnwrapError::new("Value contained None".to_string()))
    }
}

pub async fn insert_category_if_not_present(
    db: &DatabaseConnection,
    category: crate::entity::esi_categories::ActiveModel,
) -> Result<(), JagerDatabaseError> {
    match try_activevalue_to_u64(category.clone().category_id) {
        Ok(category_id) => {
            let result = insert_retry(db, category).await;
            match result {
                Ok(_) => Ok(()),
                Err(err) => {
                    if err.to_string().contains("Duplicate entry") {
                        info!("Category {} already exists", category_id);
                        Ok(())
                    } else {
                        error!("Couldn't insert category {}: {:?}", category_id, err);
                        Err(JagerDatabaseError::DBError(err))
                    }
                }
            }
        }
        Err(err) => {
            error!(
                "Couldn't insert category, unwrap on category_id failed: {:?}",
                err
            );
            Err(JagerDatabaseError::ValueUnwrapError(err))
        }
    }
}

pub async fn insert_group_if_not_present(
    db: &DatabaseConnection,
    group: crate::entity::esi_groups::ActiveModel,
) -> Result<(), JagerDatabaseError> {
    match try_activevalue_to_u64(group.clone().group_id) {
        Ok(group_id) => {
            let result = insert_retry(db, group).await;
            match result {
                Ok(_) => Ok(()),
                Err(err) => {
                    if err.to_string().contains("Duplicate entry") {
                        info!("Group {} already exists:", group_id);
                        Ok(())
                    } else {
                        error!("Couldn't insert type {}: {:?}", group_id, err);
                        Err(JagerDatabaseError::DBError(err))
                    }
                }
            }
        }
        Err(err) => {
            error!(
                "Couldn't insert group, unwrap on group_id failed: {:?}",
                err
            );
            Err(JagerDatabaseError::ValueUnwrapError(err))
        }
    }
}

pub async fn insert_type_if_not_present(
    db: &DatabaseConnection,
    esi_type: crate::entity::esi_types::ActiveModel,
) -> Result<(), JagerDatabaseError> {
    match try_activevalue_to_u64(esi_type.clone().type_id) {
        Ok(type_id) => {
            let result = insert_retry(db, esi_type).await;
            match result {
                Ok(_) => Ok(()),
                Err(err) => {
                    if err.to_string().contains("Duplicate entry") {
                        info!("Type {} already exists", type_id);
                        Ok(())
                    } else {
                        error!("Couldn't insert type {}: {:?}", type_id, err);
                        Err(JagerDatabaseError::DBError(err))
                    }
                }
            }
        }
        Err(err) => {
            error!("Couldn't insert type, unwrap on type_id failed: {:?}", err);
            Err(JagerDatabaseError::ValueUnwrapError(err))
        }
    }
}

pub async fn insert_pubchar_info_if_not_present(
    db: &DatabaseConnection,
    public_info: crate::entity::character_public_info::ActiveModel,
) -> Result<CreateOrExist, JagerDatabaseError> {
    match try_activevalue_to_u64(public_info.clone().character_id) {
        Ok(character_id) => match insert_retry(db, public_info).await {
            Ok(_) => Ok(CreateOrExist::Created),
            Err(err) => {
                if err.to_string().contains("Duplicate entry") {
                    Ok(CreateOrExist::Exists)
                } else {
                    error!("Couldn't insert pubchar_info {}: {:?}", character_id, err);
                    Err(JagerDatabaseError::DBError(err))
                }
            }
        },
        Err(err) => {
            error!(
                "Couldn't insert pubchar_info, unwrap on character_id failed: {:?}",
                err
            );
            Err(JagerDatabaseError::ValueUnwrapError(err))
        }
    }
}

pub async fn insert_alliance_if_not_present(
    db: &DatabaseConnection,
    alliance: crate::entity::alliances::ActiveModel,
) -> Result<CreateOrExist, JagerDatabaseError> {
    match try_activevalue_to_u64(alliance.clone().alliance_id) {
        Ok(alliance_id) => match insert_retry(db, alliance).await {
            Ok(_) => Ok(CreateOrExist::Created),
            Err(err) => {
                if is_duplicate_err(&err) {
                    Ok(CreateOrExist::Exists)
                } else {
                    error!("Couldn't insert alliance {}: {:?}", alliance_id, err);
                    Err(JagerDatabaseError::DBError(err))
                }
            }
        },
        Err(err) => {
            error!(
                "Couldn't insert alliance, unwrap on alliance_id failed: {:?}",
                err
            );
            Err(JagerDatabaseError::ValueUnwrapError(err))
        }
    }
}

pub async fn insert_corporation_if_not_present(
    db: &DatabaseConnection,
    corporation: crate::entity::corporations::ActiveModel,
) -> Result<CreateOrExist, JagerDatabaseError> {
    match try_activevalue_to_u64(corporation.clone().corporation_id) {
        Ok(corporation_id) => match insert_retry(db, corporation).await {
            Ok(_) => Ok(CreateOrExist::Created),
            Err(err) => {
                if is_duplicate_err(&err) {
                    Ok(CreateOrExist::Exists)
                } else {
                    error!("Couldn't insert corporation {}: {:?}", corporation_id, err);
                    Err(JagerDatabaseError::DBError(err))
                }
            }
        },
        Err(err) => {
            error!(
                "Couldn't insert corporation, unwrap on corporation_id failed: {:?}",
                err
            );
            Err(JagerDatabaseError::ValueUnwrapError(err))
        }
    }
}

pub async fn insert_corporations_if_not_present(
    db: &DatabaseConnection,
    corporations: Vec<crate::entity::corporations::ActiveModel>,
) -> Result<(), JagerDatabaseError> {
    let mut error: Option<JagerDatabaseError> = Option::None;
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
) -> Result<(), JagerDatabaseError> {
    let mut error: Option<JagerDatabaseError> = Option::None;
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
) -> Result<(), JagerDatabaseError> {
    let mut error: Option<JagerDatabaseError> = Option::None;
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
) -> Result<CreateOrExist, JagerDatabaseError> {
    match try_activevalue_to_u64(killmail.clone().killmail_id) {
        Ok(killmail_id) => match insert_retry(db, killmail).await {
            Ok(_) => Ok(CreateOrExist::Created),
            Err(err) => {
                if is_duplicate_err(&err) {
                    Ok(CreateOrExist::Exists)
                } else {
                    error!("Couldn't insert killmail {}: {:?}", killmail_id, err);
                    Err(JagerDatabaseError::DBError(err))
                }
            }
        },
        Err(err) => {
            error!(
                "Couldn't insert killmail, unwrap on killmail_id failed: {:?}",
                err
            );
            Err(JagerDatabaseError::ValueUnwrapError(err))
        }
    }
}

pub fn is_duplicate_err(err: &DbErr) -> bool {
    err.to_string().contains("Duplicate entry")
}

pub async fn insert_faction_if_not_present(
    db: &DatabaseConnection,
    faction: crate::entity::factions::ActiveModel,
) -> Result<CreateOrExist, JagerDatabaseError> {
    match try_activevalue_to_u64(faction.clone().faction_id) {
        Ok(faction_id) => match insert_retry(db, faction).await {
            Ok(_) => Ok(CreateOrExist::Created),
            Err(err) => {
                if is_duplicate_err(&err) {
                    Ok(CreateOrExist::Exists)
                } else {
                    error!("Couldn't insert faction {}: {:?}", faction_id, err);
                    Err(JagerDatabaseError::DBError(err))
                }
            }
        },
        Err(err) => {
            error!(
                "Couldn't insert faction, unwrap on faction_id failed: {:?}",
                err
            );
            Err(JagerDatabaseError::ValueUnwrapError(err))
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
