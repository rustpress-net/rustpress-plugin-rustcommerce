//! Shopping cart module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub items: Vec<CartItem>,
}
