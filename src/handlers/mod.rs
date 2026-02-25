//! RustCommerce HTTP request handlers
//!
//! Axum handler functions that extract request data, call services,
//! and return JSON responses.

pub mod product_handler;
pub mod cart_handler;
pub mod checkout_handler;
pub mod order_handler;
pub mod customer_handler;
pub mod admin_handler;
pub mod webhook_handler;
pub mod settings_handler;

pub use product_handler::*;
pub use cart_handler::*;
pub use checkout_handler::*;
pub use order_handler::*;
pub use customer_handler::*;
pub use admin_handler::*;
pub use webhook_handler::*;
pub use settings_handler::*;
