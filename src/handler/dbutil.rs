use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, DeleteItemInput, DynamoDb, DynamoDbClient, PutItemInput, QueryInput,
};
use warp::reject::custom as rejection;
use warp::Rejection;

use crate::handler::constants::{CONFIG, LOCAL, LOCAL_DYNAMODB};
use crate::handler::models::Inventory;

use super::constants::*;
use super::errors::PaymentErr;

lazy_static! {
    pub static ref DB_CLIENT: Arc<DynamoDbClient> = Arc::new(create_client());
}

fn create_client() -> DynamoDbClient {
    let region_name = match CONFIG.get_str("AWS_DEFAULT_REGION") {
        Ok(region_name) => region_name,
        Err(_) => {
            error!("server error: {}", PaymentErr::AwsRegionNotSet.to_string());
            LOCAL.to_string()
        }
    };
    let region = match region_name.as_ref() {
        LOCAL => Region::Custom {
            name: LOCAL.to_string(),
            endpoint: match CONFIG.get_str("DYNAMODB_LOCAL_URL") {
                Ok(url) => url,
                Err(_) => String::from(LOCAL_DYNAMODB),
            },
        },
        region => Region::from_str(region).unwrap(),
    };
    DynamoDbClient::new(region)
}

pub fn to_inventory(
    payment_output_item: Option<::std::collections::HashMap<String, AttributeValue>>,
) -> Inventory {
    match payment_output_item {
        Some(payment_output_map) => {
            let inventory: Inventory = serde_dynamodb::from_hashmap(payment_output_map).unwrap();
            inventory
        }
        None => {
            error!(
                "server error: {}",
                PaymentErr::InventoryNotFound.to_string()
            );
            Inventory {
                ..Default::default()
            }
        }
    }
}

pub fn inventory_to_delete_input(inventory: &Inventory) -> DeleteItemInput {
    let sku_id_key_val = AttributeValue {
        n: Some(inventory.sku.to_string()),
        ..AttributeValue::default()
    };
    DeleteItemInput {
        table_name: INVENTORY.to_string(),
        key: hashmap! {
            SKU_ID.to_string() => sku_id_key_val,
        },
        ..DeleteItemInput::default()
    }
}

pub async fn delete_inventory(
    mut form_map: HashMap<String, String>,
) -> Result<Inventory, Rejection> {
    info!("Deleting inventory: {:?}", form_map);
    let sku = form_map
        .remove("entity_type")
        .unwrap_or_else(|| panic!("{}", PaymentErr::DeleteFieldsMissing.to_string()));

    let inventory = Inventory {
        sku,
        ..Inventory::default()
    };
    let _query_params = inventory_to_delete_input(&inventory);
    let query_resp = DB_CLIENT.clone().delete_item(_query_params).await;

    match query_resp {
        Ok(_) => Ok(inventory),
        Err(err) => {
            error!("Failed to delete inventory update - {:?}", err);
            Err(rejection(PaymentErr::RusotoDynamodbError))
        }
    }
}

pub async fn get_inventory_by_sku(sku: String) -> Result<Inventory, Rejection> {
    let sku_val = AttributeValue {
        s: Some(sku.to_string()),
        ..AttributeValue::default()
    };
    let query = QueryInput {
        table_name: PAYMENT.to_string(),
        index_name: Some(CONFIG.get_str("SKU_ID_ID_INDEX").unwrap()),
        key_condition_expression: Some(format!("#{} = :{}", SKU_ID, SKU_ID)),
        expression_attribute_names: Some(hashmap! {
            format!("#{}", SKU_ID) => SKU_ID.to_string(),
        }),
        expression_attribute_values: Some(hashmap! {
            format!(":{}", SKU_ID) => sku_val
        }),
        ..QueryInput::default()
    };

    let query_resp = DB_CLIENT.clone().query(query).await;

    match query_resp {
        Ok(result) => {
            if result.count.unwrap() > i64::from(0) {
                let payment_result = result.items.unwrap().pop();
                let inventory = to_inventory(payment_result);
                Ok(inventory)
            } else {
                Err(rejection(PaymentErr::NoPreviousPayment))
            }
        }
        Err(err) => {
            error!("error trying to query payment data {}", err.to_string());
            Err(rejection(PaymentErr::RusotoDynamodbError))
        }
    }
}

pub async fn find_inventory_by_title(title: String) -> Result<Vec<Inventory>, Rejection> {
    let title_val = AttributeValue {
        s: Some(title),
        ..AttributeValue::default()
    };
    let query = QueryInput {
        table_name: PAYMENT.to_string(),
        index_name: Some(CONFIG.get_str("TITLE_INDEX").unwrap()),
        key_condition_expression: Some(format!("#{} = :{}", TITLE, TITLE)),
        expression_attribute_names: Some(hashmap! {
            format!("#{}", TITLE) => TITLE.to_string(),
        }),
        expression_attribute_values: Some(hashmap! {
            format!(":{}", TITLE) => title_val
        }),
        ..QueryInput::default()
    };

    let query_resp = DB_CLIENT.clone().query(query).await;

    match query_resp {
        Ok(result) => {
            if result.count.unwrap() > i64::from(0) {
                let inventory_list: Vec<Inventory> = result
                    .items
                    .unwrap()
                    .into_iter()
                    .map(move |inventory_result| to_inventory(Some(inventory_result)))
                    .collect();
                Ok(inventory_list)
            } else {
                Err(rejection(PaymentErr::NoPreviousPayment))
            }
        }
        Err(err) => {
            error!("error trying to query payment data {}", err.to_string());
            Err(rejection(PaymentErr::RusotoDynamodbError))
        }
    }
}

pub fn inventory_to_put_input(inventory: &Inventory) -> PutItemInput {
    let mut inventory_update: HashMap<String, AttributeValue> =
        serde_dynamodb::to_hashmap(inventory).unwrap();
    inventory_update.insert(
        String::from(SKU_ID),
        AttributeValue {
            n: Some(inventory.sku.to_string()),
            ..AttributeValue::default()
        },
    );
    PutItemInput {
        table_name: INVENTORY.to_string(),
        item: inventory_update,
        ..PutItemInput::default()
    }
}

pub async fn write_inventory_update(inventory: Inventory) -> Result<Inventory, Rejection> {
    info!("Saving inventory: {:?}", inventory);
    let _query_params = inventory_to_put_input(&inventory);
    let query_resp = DB_CLIENT.clone().put_item(_query_params).await;

    match query_resp {
        Ok(_) => Ok(inventory),
        Err(err) => {
            error!("Failed to save inventory update - {:?}", err);
            Err(rejection(PaymentErr::RusotoDynamodbError))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client() {
        let resp = std::panic::catch_unwind(|| {
            create_client();
        });
        assert!(resp.is_ok())
    }
}
