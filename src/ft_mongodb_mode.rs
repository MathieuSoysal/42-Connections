use std::error::Error;

use log::{debug, info};
use mongodb::{
    bson::{doc, Bson, Document},
    Client, Collection,
};

#[derive(Debug, PartialEq)]
pub enum Mode {
    ProfilesIndexing,
    Profiles,
    LocationsIndexing,
    Locations,
    UserEvents,
    Events,
}

impl Mode {
    pub fn as_str(&self) -> &str {
        match self {
            Mode::ProfilesIndexing => "profiles_indexing",
            Mode::Profiles => "profiles",
            Mode::LocationsIndexing => "locations_indexing",
            Mode::Locations => "locations",
            Mode::UserEvents => "user_events",
            Mode::Events => "events",
        }
    }
}

pub fn get_mode_from_str(mode: &str) -> Mode {
    match mode {
        "profiles_indexing" => Mode::ProfilesIndexing,
        "profiles" => Mode::Profiles,
        "locations_indexing" => Mode::LocationsIndexing,
        "locations" => Mode::Locations,
        "user_events" => Mode::UserEvents,
        "events" => Mode::Events,
        _ => Mode::ProfilesIndexing,
    }
}

pub async fn get_current_mode_from_mongo(client: &Client) -> Result<Mode, Box<dyn Error>> {
    info!("Fetching current mode from MongoDB.");
    let collection: Collection<Document> = client.database("application").collection("mode");
    let found_doc = collection.find_one(doc! { "_id": 1 }).await;
    if found_doc.is_err() {
        info!("Failed to find a mode in MongoDB");
        return Ok(Mode::Profiles);
    }
    let current_mode = convert_to_mode(found_doc?);
    info!("Current mode is {}", current_mode.as_str());
    Ok(current_mode)
}

pub async fn update_mode_in_mongo(client: &Client) -> Result<(), Box<dyn Error>> {
    debug!("Updating mode in MongoDB.");
    let current_mode = get_current_mode_from_mongo(client).await?;
    let next_mode = get_next_mode(current_mode);
    insert_mode_in_mongo(client, next_mode).await?;
    debug!("Updated mode in MongoDB.");
    Ok(())
}

async fn insert_mode_in_mongo(client: &Client, mode: Mode) -> Result<(), Box<dyn Error>> {
    debug!("Inserting mode in MongoDB.");
    let collection: Collection<Document> = client.database("application").collection("mode");
    collection
        .insert_one(doc! {"_id": 1, "mode": mode.as_str()})
        .await?;
    debug!("Inserted mode in MongoDB.");
    Ok(())
}

fn convert_to_mode(found_doc: Option<Document>) -> Mode {
    match found_doc {
        Some(doc) => {
            let bson_mode = doc.get("mode").unwrap();
            if let Bson::String(mode) = bson_mode {
                get_mode_from_str(mode)
            } else {
                Mode::Profiles
            }
        }
        None => Mode::Profiles,
    }
}

fn get_next_mode(current_mode: Mode) -> Mode {
    match current_mode {
        Mode::ProfilesIndexing => Mode::Profiles,
        Mode::Profiles => Mode::LocationsIndexing,
        Mode::LocationsIndexing => Mode::Locations,
        Mode::Locations => Mode::UserEvents,
        Mode::UserEvents => Mode::Events,
        Mode::Events => Mode::ProfilesIndexing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::doc;
    use mongodb::{options::ClientOptions, Client};
    use testcontainers::{
        core::IntoContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage,
    };

    pub async fn get_test_mongo_client() -> (Client, ContainerAsync<GenericImage>) {
        let container = match GenericImage::new("mongo", "latest")
            .with_exposed_port(27017.tcp())
            .start()
            .await
        {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to start MongoDB container: {}", e);
            }
        };

        // Get the host and port
        let port = match container.get_host_port_ipv4(27017).await {
            Ok(p) => p,
            Err(e) => {
                panic!("Failed to get MongoDB container port: {}", e);
            }
        };

        let client_uri = format!("mongodb://localhost:{}/", port);
        let options = ClientOptions::parse(&client_uri).await.unwrap();
        let client = Client::with_options(options).unwrap();

        let db = client.database("admin");
        for _ in 0..10 {
            match db.run_command(doc! {"ping": 1}).await {
                Ok(_) => break,
                Err(e) => {
                    eprintln!("Waiting for MongoDB to be ready: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }

        (client, container)
    }

    #[tokio::test]
    async fn test_get_current_mode_from_mongo() {
        let (client, container) = get_test_mongo_client().await;
        let mode = get_current_mode_from_mongo(&client).await.unwrap();
        container.stop().await.unwrap();
        assert_eq!(mode, Mode::Profiles);
    }

    #[tokio::test]
    async fn test_update_mode_in_mongo() {
        let (client, container) = get_test_mongo_client().await;
        update_mode_in_mongo(&client).await.unwrap();
        let mode = get_current_mode_from_mongo(&client).await.unwrap();
        container.stop().await.unwrap();
        assert_eq!(mode, Mode::LocationsIndexing);
    }
}
