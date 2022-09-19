use crate::handler::errors::PaymentErr;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::reject::custom as rejection;
use warp::Rejection;

use super::models::Inventory;

pub async fn check_inventory_validity(inventory: Inventory) -> Result<Inventory, Rejection> {
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    if inventory.expiry > since_the_epoch {
        Ok(inventory)
    } else {
        Err(rejection(PaymentErr::InventoryDetailsNotFound))
    }
}

pub async fn check_all_inventory_validity(
    inventory_list: Vec<Inventory>,
) -> Result<Vec<Inventory>, Rejection> {
    let time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let valid_inventory_list: Vec<Inventory> = inventory_list
        .into_iter()
        .filter(|inventory| inventory.expiry > time_now)
        .collect();

    if !valid_inventory_list.is_empty() {
        Ok(valid_inventory_list)
    } else {
        Err(rejection(PaymentErr::InventoryDetailsNotFound))
    }
}

