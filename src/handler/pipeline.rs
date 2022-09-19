use crate::handler::models::*;
use kafka::producer::{Producer, Record};
use schema_registry_converter::schema_registry::SubjectNameStrategy;
use schema_registry_converter::Encoder;
use serde::Serialize;
use warp::Rejection;

use crate::handler::constants::CONFIG;

//kafka-topics --zookeeper kafka:2181 --create --topic payment_events --replication-factor 1 --partitions 4
fn producer() -> Result<Producer, kafka::error::Error> {
    let kafka_url = CONFIG.get_str("KAFKA_ENDPOINT").unwrap();
    Producer::from_hosts(vec![kafka_url]).create()
}

fn _send_avro_event(item: impl Serialize, topic: String) {
    let mut encoder = Encoder::new(CONFIG.get_str("SCHEMA_REGISTRY_ENDPOINT").unwrap());
    let value_strategy =
        SubjectNameStrategy::TopicRecordNameStrategy(topic.to_string(), "value".into());
    println!(
        "Schema reg endpoint: {}",
        CONFIG.get_str("SCHEMA_REGISTRY_ENDPOINT").unwrap()
    );
    println!("Topic: {}", topic);
    let bytes_result = encoder.encode_struct(item, &value_strategy);
    match bytes_result {
        Ok(bytes) => match producer() {
            Ok(mut producer) => {
                info!("Producing to Topic: {}", topic);
                let msg = producer.send(&Record::from_value(&topic, bytes));
                match msg {
                    Ok(_) => info!("Payment service activity streamed to kafka"),
                    Err(err) => error!(
                        "Failed to post payment user event message at topic {} to kafka: {:?}",
                        topic, err
                    ),
                }
            }
            Err(err) => error!(
                "Failed to initiate producer to post payment user event to kafka: {}",
                err
            ),
        },
        Err(err) => error!("Failed to encode payment user using schema: {}", err),
    }
}

pub fn send_avro_event(item: impl Serialize, topic: String) {
    if !CONFIG.get_bool("FAIL_ON_KAFKA_DISCONNECT").unwrap() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            _send_avro_event(item, topic);
        }));
        if result.is_err() {
            error!(
                "Failed to stream event to kafka: {:?}",
                result.err().unwrap()
            );
        }
    } else {
        _send_avro_event(item, topic);
    }
}

pub async fn stream_inventory_update_event(inventory: Inventory) -> Result<Inventory, Rejection> {
    send_avro_event(
        inventory.clone(),
        CONFIG.get_str("INVENTORY_UPDATED_TOPIC").unwrap(),
    );
    Ok(inventory)
}
