use std::error::Error;

use log::{error, info};
use mongodb::{
    bson::{doc, Document},
    Client,
};

pub async fn insert_profiles_index_in_mongodb(
    client: &Client,
    profiles_node: &serde_json::Value,
) -> Result<usize, Box<dyn Error>> {
    info!("Inserting profiles index in MongoDB.");
    let locations = map_profiles_index_to_bson_documents(profiles_node).await;
    let nb_locations = locations.len();
    insert_profiles_index_into_mongodb(client, locations).await;
    Ok(nb_locations)
}

async fn map_profiles_index_to_bson_documents(profiles_node: &serde_json::Value) -> Vec<Document> {
    let locations = profiles_node
        .as_array()
        .unwrap()
        .iter()
        .map(|location_node| convert_json_profile_to_bson(location_node));
    let locations = futures::future::join_all(locations).await;
    locations
}

async fn convert_json_profile_to_bson(profile_node: &serde_json::Value) -> Document {
    let bson_value = mongodb::bson::to_bson(profile_node).unwrap();
    if let mongodb::bson::Bson::Document(mut doc) = bson_value {
        return doc! {"_id": doc.remove("id").unwrap()};
    } else {
        error!("Expected a document but got a different BSON type.");
        panic!("Expected a document but got a different BSON type.");
    }
}

async fn insert_profiles_index_into_mongodb(client: &Client, locations: Vec<Document>) {
    let result = client
        .database("application")
        .collection::<Document>("profiles_index")
        .insert_many(locations)
        .ordered(false)
        .await;
    if let Err(e) = result {
        error!("Failed to insert locations in MongoDB: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::doc;
    use mongodb::{options::ClientOptions, Client};
    use serde_json::json;
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
    async fn test_insert_profiles_index_in_mongodb() {
        let (client, container) = get_test_mongo_client().await;

        // Prepare test data
        let profiles_node = &json!([
          {
            "id": 39962,
            "email": "malallai@student.42.fr",
            "login": "malallai",
            "first_name": "Malo",
            "last_name": "Allain",
            "usual_full_name": "Malo Allain",
            "usual_first_name": null,
            "url": "https://api.intra.42.fr/v2/users/malallai",
            "phone": "hidden",
            "displayname": "Malo Allain",
            "kind": "student",
            "image": {
              "link": "https://cdn.intra.42.fr/users/39a641ed152b654cfbff5c5864eb05c1/malallai.jpg",
              "versions": {
                "large": "https://cdn.intra.42.fr/users/a818d7a54298d333411557d0b55b61b3/large_malallai.jpg",
                "medium": "https://cdn.intra.42.fr/users/53691acd1e0ea75b782ddbe121c17423/medium_malallai.jpg",
                "small": "https://cdn.intra.42.fr/users/617de91b59fd7e59ecaf6470a4b37645/small_malallai.jpg",
                "micro": "https://cdn.intra.42.fr/users/6f74b46e2016b0e6c41fa05b5952e17c/micro_malallai.jpg"
              }
            },
            "staff?": false,
            "correction_point": 4,
            "pool_month": "august",
            "pool_year": "2018",
            "location": null,
            "wallet": 290,
            "anonymize_date": "2025-10-24T00:00:00.000+02:00",
            "data_erasure_date": "2025-10-24T00:00:00.000+02:00",
            "created_at": "2018-07-17T08:57:33.128Z",
            "updated_at": "2022-09-27T18:48:28.207Z",
            "alumnized_at": null,
            "alumni?": false,
            "active?": true
          },
          {
            "id": 26134,
            "email": "asandolo@student.42.fr",
            "login": "asandolo",
            "first_name": "Alexandre",
            "last_name": "Sandolo",
            "usual_full_name": "Alexandre Sandolo",
            "usual_first_name": null,
            "url": "https://api.intra.42.fr/v2/users/asandolo",
            "phone": "hidden",
            "displayname": "Alexandre Sandolo",
            "kind": "student",
            "image": {
              "link": "https://cdn.intra.42.fr/users/e6ad4cc0cd6b69c9f18cf7fb8fefda22/asandolo.jpg",
              "versions": {
                "large": "https://cdn.intra.42.fr/users/5100f27887ee3dc6c864b618ddb44b1f/large_asandolo.jpg",
                "medium": "https://cdn.intra.42.fr/users/df044745577b908566f1cdca3e927c23/medium_asandolo.jpg",
                "small": "https://cdn.intra.42.fr/users/2946d4da689a314b6ef6f090ce724a1b/small_asandolo.jpg",
                "micro": "https://cdn.intra.42.fr/users/ff75319aa7fa4332336edf9216c51e0d/micro_asandolo.jpg"
              }
            },
            "staff?": false,
            "correction_point": 4,
            "pool_month": "july",
            "pool_year": "2017",
            "location": null,
            "wallet": 1776,
            "anonymize_date": "2025-10-11T00:00:00.000+02:00",
            "data_erasure_date": "2025-10-11T00:00:00.000+02:00",
            "created_at": "2017-06-22T11:42:39.426Z",
            "updated_at": "2022-09-27T19:20:19.605Z",
            "alumnized_at": null,
            "alumni?": false,
            "active?": true
          },
          {
            "id": 20152,
            "email": "clabouri@student.42.fr",
            "login": "clabouri",
            "first_name": "Charles",
            "last_name": "Labourier",
            "usual_full_name": "Charles Labourier",
            "usual_first_name": null,
            "url": "https://api.intra.42.fr/v2/users/clabouri",
            "phone": "hidden",
            "displayname": "Charles Labourier",
            "kind": "student",
            "image": {
              "link": "https://cdn.intra.42.fr/users/17dd9c4ee21baf1e2b259e8de08adee2/clabouri.jpg",
              "versions": {
                "large": "https://cdn.intra.42.fr/users/d310252790c850b2fa3158ad4d97ac17/large_clabouri.jpg",
                "medium": "https://cdn.intra.42.fr/users/07b3b6ef492df6ed2148b5c6f4fb5f7d/medium_clabouri.jpg",
                "small": "https://cdn.intra.42.fr/users/a28d995d70d99b439cbcf38852b197a1/small_clabouri.jpg",
                "micro": "https://cdn.intra.42.fr/users/de2ebeba74c5ff018f47f124cb84568b/micro_clabouri.jpg"
              }
            },
            "staff?": false,
            "correction_point": 3,
            "pool_month": "august",
            "pool_year": "2016",
            "location": null,
            "wallet": 525,
            "anonymize_date": null,
            "data_erasure_date": null,
            "created_at": "2016-07-19T15:43:15.282Z",
            "updated_at": "2022-09-19T15:36:06.910Z",
            "alumnized_at": "2021-09-28T17:06:21.597Z",
            "alumni?": true,
            "active?": true
          },
          {
            "id": 14819,
            "email": "nkrouglo@student.42.fr",
            "login": "nkrouglo",
            "first_name": "Natalia",
            "last_name": "Krouglov",
            "usual_full_name": "Natalia Krouglov",
            "usual_first_name": null,
            "url": "https://api.intra.42.fr/v2/users/nkrouglo",
            "phone": "hidden",
            "displayname": "Natalia Krouglov",
            "kind": "student",
            "image": {
              "link": "https://cdn.intra.42.fr/users/04fc6c399a81d114bd479dc3a5b90ea4/nkrouglo.jpg",
              "versions": {
                "large": "https://cdn.intra.42.fr/users/594b01edc55f87bf4918a861823cc448/large_nkrouglo.jpg",
                "medium": "https://cdn.intra.42.fr/users/3b1dc67a9b58c7e8b8bd072fa9c4aa6e/medium_nkrouglo.jpg",
                "small": "https://cdn.intra.42.fr/users/2a8263df5f95515917148068fa82eb3f/small_nkrouglo.jpg",
                "micro": "https://cdn.intra.42.fr/users/7e52b8f556ada1e2cf23d5c6d6515094/micro_nkrouglo.jpg"
              }
            },
            "staff?": false,
            "correction_point": 3,
            "pool_month": "july",
            "pool_year": "2015",
            "location": null,
            "wallet": 240,
            "anonymize_date": "2024-01-08T00:00:00.000+01:00",
            "data_erasure_date": "2024-01-08T00:00:00.000+01:00",
            "created_at": "2016-01-20T00:38:39.938Z",
            "updated_at": "2022-09-19T15:35:56.534Z",
            "alumnized_at": null,
            "alumni?": false,
            "active?": false
          }
        ]);

        // Call the function to test
        let result = insert_profiles_index_in_mongodb(&client, profiles_node).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);

        // Verify data insertion
        let collection = client
            .database("application")
            .collection::<Document>("profiles_index");
        let count = collection.count_documents(doc! {}).await.unwrap();
        container.stop().await.unwrap();
        assert_eq!(count, 4);
    }
}
