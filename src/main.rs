#![feature(test)]
extern crate test;

use fetching::TIME_BETWEEN_REQUESTS;
use ft_api::generate_access_token;
use ft_mongodb::fetch_current_index;
use log::{debug, error, info};
use oauth2::AccessToken;
use std::{env, error::Error, time::Duration};
use tokio::time::{sleep_until, Instant};

pub mod fetching;
pub mod ft_api;
pub mod ft_mongodb;

const NB_MINUTES: u32 = 10;
const NB_FETCH: u32 = (NB_MINUTES * 60) / TIME_BETWEEN_REQUESTS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Starting 42 analytics.");
    let (client, secret_key_profil, secret_key_location) = initialize_variables().await?;
    let user_id = fetch_current_index(&client, NB_FETCH * 2).await.unwrap();
    let mut i = user_id;
    while i < user_id + (NB_FETCH * 2) {
        let current = Instant::now();
        futures::future::try_join(
            fetching::fetch_profil_from_42_to_mongo(&client, i.clone(), &secret_key_location),
            fetching::fetch_profil_from_42_to_mongo(&client, i.clone() + 1, &secret_key_profil),
        )
        .await?;
        i += 2;
        sleep_until(current + Duration::from_secs(TIME_BETWEEN_REQUESTS.into())).await;
    }
    info!("42 analytics finished.");
    Ok(())
}

async fn initialize_variables(
) -> Result<(mongodb::Client, AccessToken, AccessToken), Box<dyn Error>> {
    let mongodb_uri = get_var_env("MONGODB_URI");
    let client = ft_mongodb::connect_to_mongodb(&mongodb_uri).await;
    let secret_key_profil = generate_access_token(
        &get_var_env("SECRET_ID_PROFIL"),
        &get_var_env("SECRET_KEY_PROFIL"),
    )
    .await?;
    let secret_key_location = generate_access_token(
        &get_var_env("SECRET_ID_LOCATION"),
        &get_var_env("SECRET_KEY_LOCATION"),
    )
    .await?;
    Ok((client, secret_key_profil, secret_key_location))
}

fn get_var_env(env_name: &str) -> String {
    match env::var(env_name) {
        Ok(val) => {
            debug!("Retrieved {} from environment.", env_name);
            val
        }
        Err(e) => {
            error!("{} environment variable not set: {}", env_name, e);
            panic!();
        }
    }
}
