#![feature(test)]
extern crate test;

use fetching_event::double_fetch_event_from_42_to_mongo;
use fetching_profile::TIME_BETWEEN_REQUESTS;
use ft_api::generate_access_token;
use ft_mongodb_mode::Mode;
use log::{debug, error, info};
use oauth2::AccessToken;
use tokio::time::{sleep_until, Instant};
use std::{env, error::Error, time::Duration};

pub mod fetching_profile;
pub mod fetching_event_participation;
pub mod fetching_locations;
pub mod fetching_event;
pub mod ft_api;
pub mod ft_mongodb;
pub mod ft_mongodb_app_indexor;
pub mod ft_mongodb_events_participation;
pub mod ft_mongodb_last_update;
pub mod ft_mongodb_locations;
pub mod ft_mongodb_mode;
pub mod ft_mongodb_profile_indexer;
pub mod ft_mongodb_profiles;
pub mod ft_mongodb_events;

pub const NB_MINUTES: u32 = 10;
pub const NB_FETCH: u32 = (NB_MINUTES * 60) / TIME_BETWEEN_REQUESTS;
pub const MAX_INDEX: u32 = 207864;

pub const CURRENT_MODE: Mode = Mode::UserEvents;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Starting 42 analytics.");
    let (client, api_key_1, api_key_2) = initialize_variables().await?;
    for _ in 0..NB_FETCH {
        let current = Instant::now();
        double_fetch_event_from_42_to_mongo(&client, &api_key_1, &api_key_2).await?;
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
