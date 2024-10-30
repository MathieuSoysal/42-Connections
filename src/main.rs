#![feature(test)]
extern crate test;

use ft_api::generate_access_token;
use ft_mongodb::fetch_current_index;
use log::{debug, error, info};
use std::{env, error::Error};
use tokio::time::sleep;

pub mod fetching;
pub mod ft_api;
pub mod ft_mongodb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Starting 42 analytics.");
    let mongodb_uri = get_var_env("MONGODB_URI").unwrap();
    let client = ft_mongodb::connect_to_mongodb(&mongodb_uri).await.unwrap();
    let secret_key_profil = generate_access_token(
        get_var_env("SECRET_ID_PROFIL").unwrap().as_str(),
        get_var_env("SECRET_KEY_PROFIL").unwrap().as_str(),
    )
    .await?;
    let secret_key_location = generate_access_token(
        get_var_env("SECRET_ID_LOCATION").unwrap().as_str(),
        get_var_env("SECRET_KEY_LOCATION").unwrap().as_str(),
    )
    .await?;
    let user_id = fetch_current_index(&client).await.unwrap();
    for i in user_id..user_id + 20 {
        info!("Fetching data for user_id: {}", i);
        fetching::fetch_profil_from_42_to_mongo(&client, i, &secret_key_profil).await?;
        fetching::fetch_location_from_42_to_mongo(&client, i, &secret_key_location).await?;
        sleep(std::time::Duration::from_secs(3)).await;
    }
    info!("42 analytics finished.");
    Ok(())
}

fn get_var_env(env_name: &str) -> Result<String, env::VarError> {
    match env::var(env_name) {
        Ok(val) => {
            debug!("Retrieved {} from environment.", env_name);
            Ok(val)
        }
        Err(e) => {
            error!("{} environment variable not set: {}", env_name, e);
            Err(e)
        }
    }
}
