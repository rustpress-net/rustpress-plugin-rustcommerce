//! Order processing module

use serde::{Deserialize, Serialize};
use crate::cart::CartItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub items: Vec<CartItem>,
    pub status: OrderStatus,
    pub total: f64,
}
