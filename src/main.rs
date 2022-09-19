#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
extern crate serde;
#[macro_use]
extern crate warp;

use std::env;
use std::net::SocketAddr;

use handler::service_routes;

mod handler;

#[tokio::main]
async fn main() {
    stackdriver_logger::init();

    let host = env::var("SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("SERVICE_PORT")
        .unwrap_or_else(|_| "9999".to_string())
        .parse::<u16>()
        .unwrap();
    let address = format!("{host}:{port}", host = host, port = port)
        .parse::<SocketAddr>()
        .expect("Invalid socket address");

    warp::serve(service_routes()).run(address).await;
}
