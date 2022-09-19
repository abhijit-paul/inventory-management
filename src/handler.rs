use std::str;

use serde_json::{self, json};
use warp::http::StatusCode;
use warp::Filter;

use crate::handler::models::Inventory;

mod constants;
mod dbutil;
mod errors;
mod inventory;
mod models;
mod pipeline;
mod ready;

pub fn service_routes() -> impl Filter<Extract = impl warp::Reply> + Clone {
    let alive = warp::get()
        .and(warp::path("alive"))
        .and(warp::path::end())
        .map(|| warp::reply::with_status("OK", StatusCode::OK));

    let ready = warp::get()
        .and(warp::path("ready"))
        .and(warp::path::end())
        .and_then(ready::readiness_check)
        .map(|(success, resp)| {
            let code = if success {
                StatusCode::OK
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            warp::reply::with_status(
                warp::reply::json(&json!({
                    "code": code.as_u16(),
                    "message": resp
                })),
                code,
            )
        });

    let check_inventory = warp::get()
        .and(path!("api" / "inventory").and(warp::path::end()))
        .and(warp::query::<String>())
        .and_then(dbutil::get_inventory_by_sku)
        .and_then(inventory::check_inventory_validity)
        .map(|res: Inventory| warp::reply::json(&res));

    let query_inventory = warp::get()
        .and(path!("api" / "find" / "inventory").and(warp::path::end()))
        .and(warp::query::<String>())
        .and_then(dbutil::find_inventory_by_title)
        .and_then(inventory::check_all_inventory_validity)
        .map(|res: Vec<Inventory>| warp::reply::json(&res));

    let delete_inventory = warp::delete()
        .and(path!("api" / "inventory").and(warp::path::end()))
        .and(warp::body::content_length_limit(1024 * 32).and(warp::body::form()))
        .and_then(dbutil::delete_inventory)
        .and_then(pipeline::stream_inventory_update_event)
        .map(|inventory: Inventory| {
            warp::reply::with_status(
                format!(
                    "Deleted inventory of {} to {}{} with expiry {}",
                    inventory.sku, inventory.currency_symbol, inventory.amount, inventory.expiry,
                ),
                StatusCode::OK,
            )
        });

    let add_inventory = warp::post()
        .and(path!("api" / "inventory").and(warp::path::end()))
        .and(warp::body::content_length_limit(1024 * 32).and(warp::body::form()))
        .and_then(dbutil::write_inventory_update)
        .and_then(pipeline::stream_inventory_update_event)
        .map(|inventory: Inventory| {
            warp::reply::with_status(
                format!(
                    "Updated inventory of {} to {}{} with expiry {}",
                    inventory.sku, inventory.currency_symbol, inventory.amount, inventory.expiry,
                ),
                StatusCode::OK,
            )
        });

    alive
        .or(ready)
        .or(check_inventory)
        .or(query_inventory)
        .or(delete_inventory)
        .or(add_inventory)
        .recover(errors::customize_error)
        .with(warp::log("inventory_management"))
}
