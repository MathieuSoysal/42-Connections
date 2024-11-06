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
) -> Result<(), Box<dyn Error>> {
    info!("Inserting location in MongoDB.");
    let locations = locations_node
        .as_array()
        .unwrap()
        .iter()
        .map(|location_node| insert_location_in_mongodb(client, location_node, user_id));
    futures::future::join_all(locations).await;
    Ok(())
}

pub async fn get_an_user_id_and_page_number(client: &Client) -> Result<(i64, i32), Box<dyn Error>> {
    info!("Fetching current index from MongoDB.");
    let collection: Collection<Document> =
        client.database("application").collection("locations_index");
    let result = collection.find_one_and_delete(doc! {}).await.or_else(|e| {
        error!("Failed to find a location index from MongoDB: {}", e);
        Err(e)
    })?;
    if let Some(doc) = result {
        let (user_id, page_number) = match parse_location_index(doc) {
            Ok(value) => value,
            Err(value) => return value,
        };
        info!("Current location index is {}", user_id);
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
    info!("Inserting location index in MongoDB.");
    let collection: Collection<Document> =
        client.database("application").collection("locations_index");
    let doc = doc! { "_id": user_id, "page_number": page_number };
    collection
        .update_one(doc! {"_id": user_id}, doc)
        .upsert(true)
        .await
        .or_else(|e| {
            error!("Failed to insert location index in MongoDB: {}", e);
            Err(e)
        })?;
    info!("Location index inserted in MongoDB.");
    Ok(())
}

async fn insert_location_in_mongodb(
    client: &Client,
    location_node: &serde_json::Value,
    user_id: i64,
) -> Result<(), Box<dyn Error>> {
    let collection: Collection<Document> = client.database("42").collection("locations");
    let bson_value = mongodb::bson::to_bson(location_node).unwrap();
    if let mongodb::bson::Bson::Document(mut doc) = bson_value {
        doc.insert("user_id", user_id);
        let location_id = doc.get("id").unwrap().as_i64().unwrap();
        doc.insert("_id", location_id);
        doc.remove("user");
        doc.remove("project");
        let filter = doc! { "_id": location_id };
        collection
            .replace_one(filter, doc)
            .upsert(true)
            .await
            .or_else(|e| {
                error!(
                    "Failed to insert location {} in MongoDB: {}",
                    location_id, e
                );
                Err(e)
            })?;
        debug!("Location {} inserted in MongoDB.", location_id);
    } else {
        error!("Expected a document but got a different BSON type.");
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    // Add your test cases here

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
