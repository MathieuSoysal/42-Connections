use std::error::Error;

use log::{error, info};
use mongodb::{bson::Document, Client};

pub async fn insert_user_locations_in_mongodb(
    client: &Client,
    user_id: i64,
    locations_node: &serde_json::Value,
) -> Result<usize, Box<dyn Error>> {
    info!("Inserting locations in MongoDB for user {}.", user_id);
    let locations = map_locations_to_bson_documents(locations_node, user_id).await;
    let nb_locations = locations.len();
    insert_user_locations_into_mongodb(client, locations).await;
    Ok(nb_locations)
}

async fn convert_json_location_to_bson(
    location_node: &serde_json::Value,
    user_id: i64,
) -> Document {
    let bson_value = mongodb::bson::to_bson(location_node).unwrap();
    if let mongodb::bson::Bson::Document(mut doc) = bson_value {
        doc.insert("user_id", user_id);
        let location_id = doc.get_i64("id").unwrap();
        doc.insert("_id", location_id);
        doc.remove("user");
        doc.remove("project");
        let doc = doc;
        return doc;
    } else {
        error!("Expected a document but got a different BSON type.");
        panic!("Expected a document but got a different BSON type.");
    }
}

async fn insert_user_locations_into_mongodb(client: &Client, locations: Vec<Document>) {
    let result = client
        .database("42")
        .collection::<Document>("locations")
        .insert_many(locations)
        .ordered(false)
        .await;
    if let Err(e) = result {
        error!("Failed to insert locations in MongoDB: {}", e);
    }
}

async fn map_locations_to_bson_documents(
    locations_node: &serde_json::Value,
    user_id: i64,
) -> Vec<Document> {
    let locations = locations_node
        .as_array()
        .unwrap()
        .iter()
        .map(|location_node| convert_json_location_to_bson(location_node, user_id));
    let locations = futures::future::join_all(locations).await;
    locations
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::doc;
    use mongodb::{options::ClientOptions, Client};
    use serde_json::json;
    use std::error::Error;
    use testcontainers::{
        core::IntoContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage,
    };

    // Helper function to get a test MongoDB client using testcontainers
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
    async fn test_insert_location_in_mongodb() -> Result<(), Box<dyn Error>> {
        let (client, container) = get_test_mongo_client().await;
        let user_id = 42;
        let locations = json!([
            {
                "id": 1,
                "name": "Paris",
                "latitude": 48.8566,
                "longitude": 2.3522,
                "user": "42",
                "project": "42"
            },
            {
                "id": 2,
                "name": "London",
                "latitude": 51.5074,
                "longitude": 0.1278,
                "user": "42",
                "project": "42"
            }
        ]);
        let len = insert_user_locations_in_mongodb(&client, user_id, &locations).await?;
        assert_eq!(len, 2);
        container.stop().await?;
        Ok(())
    }
}
