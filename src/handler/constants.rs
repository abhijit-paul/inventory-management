use std::sync::Arc;

use config::{Config, Environment, File, FileFormat};

fn generate_config_from_file_and_env() -> Config {
    let mut config = Config::new();
    config
        .merge(File::new("Settings", FileFormat::Toml).required(false))
        .unwrap();
    config.merge(Environment::new()).unwrap();
    config
}

lazy_static! {
    pub static ref CONFIG: Arc<Config> = Arc::new(generate_config_from_file_and_env());
}

pub const LOCAL: &str = "local";
pub const LOCAL_DYNAMODB: &str = "localhost:8000";

pub const CURRENCY_AFFIX_SIDE: &str = "left";

//Pricing table details
pub const INVENTORY: &str = "inventory";

pub const SKU_ID: &str = "sku_id";
pub const TITLE: &str = "title";

//payment table details
pub const PAYMENT: &str = "payment-details";
