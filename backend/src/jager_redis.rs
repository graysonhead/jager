use crate::stats_processing::CharacterStats;
use bb8_redis::bb8::PooledConnection;
use bb8_redis::{
    bb8,
    RedisConnectionManager,
};
use dotenv::dotenv;
use redis::{ErrorKind};
use serde_json;
use std::env;
const EXPIRE_INTERVAL: usize = 14400;

pub async fn get_redis_pool(url: String) -> bb8_redis::bb8::Pool<RedisConnectionManager> {
    let manager = RedisConnectionManager::new(url).unwrap();
    bb8::Pool::builder().build(manager).await.unwrap()
}

pub async fn init_redis_connection() -> Option<redis::aio::Connection> {
    dotenv().ok();
    let redis_password = env::var("REDIS_AUTH_PW");
    match env::var("REDIS_URL") {
        Ok(urlval) => {
            let mut con = get_redis_client(urlval).await;
            if redis_password.is_ok() {
                redis::cmd("Auth")
                    .arg("jager18")
                    .query_async::<redis::aio::Connection, String>(&mut con)
                    .await
                    .expect("Authentication to redis failed");
            }
            Some(con)
        }
        Err(_) => None,
    }
}

async fn get_redis_client(url: String) -> redis::aio::Connection {
    let client = redis::Client::open(url).expect("Failed to connect to redis server");
    client
        .get_async_connection()
        .await
        .expect("Failed to get async connection to redis")
}

pub async fn check_cache_character_stats(
    mut conn: &mut PooledConnection<'_, RedisConnectionManager>,
    character_name: &String,
) -> Option<CharacterStats> {
    match redis::cmd("GET")
        .arg(character_name)
        .query_async::<redis::aio::Connection, String>(&mut conn)
        .await
    {
        Ok(result_string) => {
            let stats_result: Result<CharacterStats, serde_json::Error> =
                serde_json::from_str(&result_string);
            match stats_result {
                Ok(stats) => {
                    info!("Cache hit for {}", character_name);
                    Some(stats)
                }
                Err(e) => {
                    error!(
                        "Could not deserialize cache result for {}: {:?}",
                        character_name, e
                    );
                    None
                }
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::TypeError => {
                info!("Cache miss for {}", character_name);
                None
            }
            _ => {
                error!("Failed to query chache for {}: {:?}", character_name, e);
                None
            }
        },
    }
}

pub async fn cache_character_stats(
    conn: &mut PooledConnection<'_, RedisConnectionManager>,
    character_name: &String,
    info_object: &CharacterStats,
) {
    match serde_json::to_string(&info_object) {
        Ok(json_string) => {
            redis::cmd("SET")
                .arg(&character_name)
                .arg(json_string)
                .arg("EX")
                .arg(EXPIRE_INTERVAL)
                .query_async::<redis::aio::Connection, String>(conn)
                .await
                .unwrap();
            info!("Added stats to cache for {}", character_name);
        }
        Err(e) => {
            error!(
                "Caching {} failed, couldn't serialize stats: {:?}",
                character_name, e
            );
        }
    }
}
