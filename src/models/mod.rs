//! RustCommerce data models
//!
//! All domain entities used throughout the plugin. Models derive
//! `Serialize`/`Deserialize` for API transport and `sqlx::FromRow`
//! for database hydration.

pub mod product;
pub mod category;
pub mod cart;
pub mod order;
pub mod customer;
pub mod payment;
pub mod shipping;
pub mod tax;
pub mod coupon;
pub mod review;

pub use product::*;
pub use category::*;
pub use cart::*;
pub use order::*;
pub use customer::*;
pub use payment::*;
pub use shipping::*;
pub use tax::*;
pub use coupon::*;
pub use review::*;
