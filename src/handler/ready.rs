use serde_json::{self, json};
use warp::Rejection;

use crate::handler::constants::CONFIG;

pub async fn readiness_check() -> Result<(bool, String), Rejection> {
    let dynamodb_url = match CONFIG.get_str("AWS_DEFAULT_REGION").unwrap().as_ref() {
        "local" => format!("{}/shell", CONFIG.get_str("DYNAMODB_LOCAL_URL").unwrap()),
        region => format!("https://dynamodb.{}.amazonaws.com", region),
    };
    let dynamodb_response = reqwest::Client::new()
        .get(&dynamodb_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();
    let dynamodb_ready = dynamodb_response.status().is_success();

    let kafka_response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap()
        .get(&CONFIG.get_str("SCHEMA_REGISTRY_ENDPOINT").unwrap())
        .send()
        .await;

    let kafka_ready = match kafka_response {
        Ok(_) => true,
        Err(err) => {
            error!(
                "readiness_check: kafka_ready: Error connecting to schema registry: {}",
                err
            );
            false
        }
    };

    if dynamodb_ready && kafka_ready {
        Ok((
            true,
            json!({
                "dynamodb": dynamodb_ready,
                "kafka_reachable": kafka_ready,
            })
            .to_string(),
        ))
    } else {
        Ok((false, "readiness_check: Readiness Check failed".to_string()))
    }
}
