use std::error::Error;

use log::{debug, error, info};
use mongodb::{
    bson::{doc, Bson, Document},
    Client, Collection,
};

pub async fn get_an_user_id_and_page_number(
    client: &Client,
    collection_name: &str,
) -> Result<(i64, i32), Box<dyn Error>> {
    let collection: Collection<Document> =
        client.database("application").collection(collection_name);
    let result = collection.find_one_and_delete(doc! {}).await.or_else(|e| {
        error!(
            "Failed to find a index in {} from MongoDB: {}",
            collection_name, e
        );
        error!("Maybe all index have been fetched.");
        Err(e)
    })?;
    if let Some(doc) = result {
        let (user_id, page_number) = match parse_user_page_doc(doc) {
            Ok(value) => value,
            Err(value) => return value,
        };
        info!(
            "Current index is for user {} and page {}",
            user_id, page_number
        );
        return Ok((user_id, page_number));
    }
    error!("Failed to fetch current index in collection {} from MongoDB.", collection_name);
    Err("Failed to fetch current index from MongoDB.".into())
}

pub async fn insert_user_id_and_page_number(
    client: &Client,
    user_id: i64,
    page_number: i32,
    collection_name: &str,
) -> Result<(), Box<dyn Error>> {
    let collection: Collection<Document> =
        client.database("application").collection(collection_name);
    collection
        .replace_one(
            doc! {"_id": user_id},
            doc! {"_id": user_id, "page_number": page_number},
        )
        .upsert(true)
        .await
        .or_else(|e| {
            error!(
                "Failed to insert index for user {} in MongoDB: {}",
                user_id, e
            );
            Err(e)
        })?;
    info!(
        "Index for user {} and page {} inserted in MongoDB.",
        user_id, page_number
    );
    Ok(())
}

fn parse_user_page_doc(doc: Document) -> Result<(i64, i32), Result<(i64, i32), Box<dyn Error>>> {
    debug!("Found a index {:?} in MongoDB.", doc);
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
    use super::*;
    use mongodb::bson::doc;
    use mongodb::{options::ClientOptions, Client};
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
    async fn test_get_an_user_id_and_page_number() -> Result<(), Box<dyn Error>> {
        let (client, container) = get_test_mongo_client().await;
        let user_id = 42;
        let page_number = 1;
        insert_user_id_and_page_number(&client, user_id, page_number, "locations_index").await?;
        let (found_user_id, found_page_number) =
            get_an_user_id_and_page_number(&client, "locations_index").await?;
        assert_eq!(found_user_id, user_id);
        assert_eq!(found_page_number, page_number);
        container.stop().await?;
        Ok(())
    }

    #[test]
    fn test_parse_location_index_with_i32() {
        use super::*;
        let doc = doc! { "_id": 42 as i32, "page_number": 1 };
        assert_eq!(parse_user_page_doc(doc).unwrap(), (42, 1));
    }

    #[test]
    fn test_parse_location_index_with_i64() {
        use super::*;
        let doc = doc! { "_id": 42 as i64, "page_number": 1 };
        assert_eq!(parse_user_page_doc(doc).unwrap(), (42, 1));
    }
}
