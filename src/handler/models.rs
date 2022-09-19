use serde::{Deserialize, Serialize};

use super::constants;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Inventory {
    pub sku: String,

    pub title: String,
    pub description: f32,

    pub quantity: u64,

    pub expiry: u128, // Timestamp

    //Take care of multiple currency types.
    // Some currency types have code on left side like $10.99
    // Some currency types have code on right side like Dinar
    #[serde(rename = "currencyCode")]
    pub currency_code: String,

    #[serde(rename = "currency")]
    pub currency_symbol: String,

    #[serde(rename = "currencyAffixSide")]
    #[serde(default = "default_currency_affix_side")]
    pub currency_affix_side: String,

    pub amount: u64,
}

fn default_currency_affix_side() -> String {
    constants::CURRENCY_AFFIX_SIDE.to_string()
}
