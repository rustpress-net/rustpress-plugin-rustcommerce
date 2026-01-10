//! Product management module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: u32,
}
