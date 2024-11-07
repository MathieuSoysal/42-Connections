use std::error::Error;

use log::{debug, error, info};
use mongodb::{
    bson::{doc, Bson, Document},
    Client, Collection,
};

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

pub async fn get_an_user_id_and_page_number(client: &Client) -> Result<(i64, i32), Box<dyn Error>> {
    let collection: Collection<Document> =
        client.database("application").collection("locations_index");
    let result = collection.find_one_and_delete(doc! {}).await.or_else(|e| {
        error!("Failed to find a location index from MongoDB: {}", e);
        error!("Maybe all locations have been fetched.");
        Err(e)
    })?;
    if let Some(doc) = result {
        let (user_id, page_number) = match parse_location_index(doc) {
            Ok(value) => value,
            Err(value) => return value,
        };
        info!(
            "Current location index is for user {} and page {}",
            user_id, page_number
        );
        return Ok((user_id, page_number));
    }
    error!("Failed to fetch current index from MongoDB.");
    Err("Failed to fetch current index from MongoDB.".into())
}

pub async fn insert_user_id_and_page_number(
    client: &Client,
    user_id: i64,
    page_number: i32,
) -> Result<(), Box<dyn Error>> {
    let collection: Collection<Document> =
        client.database("application").collection("locations_index");
    collection
        .replace_one(
            doc! {"_id": user_id},
            doc! {"_id": user_id, "page_number": page_number},
        )
        .upsert(true)
        .await
        .or_else(|e| {
            error!(
                "Failed to insert location index for user {} in MongoDB: {}",
                user_id, e
            );
            Err(e)
        })?;
    info!(
        "Location index for user {} and page {} inserted in MongoDB.",
        user_id, page_number
    );
    Ok(())
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

fn parse_location_index(doc: Document) -> Result<(i64, i32), Result<(i64, i32), Box<dyn Error>>> {
    debug!("Found a location index {:?} in MongoDB.", doc);
    let user_id = match doc.get("_id") {
        Some(Bson::Int64(id)) => *id,
        Some(Bson::Int32(id)) => *id as i64,
        _ => return Err(Err("Field '_id' does not have the expected type".into())),
    };
    let page_number = match doc.get("page_number") {
        Some(Bson::Int32(page)) => *page,
        Some(Bson::Int64(page)) => *page as i32,
        _ => {
            return Err(Err(
                "Field 'page_number' does not have the expected type".into()
            ))
        }
    };
    Ok((user_id, page_number))
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
    use mongodb::{options::ClientOptions, Client};
    use serde_json::json;
    use std::error::Error;
    use testcontainers::{core::IntoContainerPort, runners::AsyncRunner, GenericImage};

    // Helper function to get a test MongoDB client using testcontainers
    async fn get_test_mongo_client() -> Client {
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

        client
    }

    #[tokio::test]
    async fn test_insert_location_in_mongodb() -> Result<(), Box<dyn Error>> {
        let client = get_test_mongo_client().await;
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
        Ok(())
    }

    #[test]
    fn test_parse_location_index_with_i32() {
        use super::*;
        let doc = doc! { "_id": 42 as i32, "page_number": 1 };
        assert_eq!(parse_location_index(doc).unwrap(), (42, 1));
    }

    #[test]
    fn test_parse_location_index_with_i64() {
        use super::*;
        let doc = doc! { "_id": 42 as i64, "page_number": 1 };
        assert_eq!(parse_location_index(doc).unwrap(), (42, 1));
    }
}
