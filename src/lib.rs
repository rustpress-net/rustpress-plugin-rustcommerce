//! # RustCommerce
//!
//! Enterprise e-commerce plugin for [RustPress](https://rustpress.net) CMS.
//!
//! RustCommerce provides a complete e-commerce backend including:
//!
//! - **Product catalogue** with categories, variants, and images
//! - **Shopping cart** with session and customer-based carts
//! - **Checkout workflow** with address validation and order creation
//! - **Order management** with status workflow and history
//! - **Payment processing** via Stripe with webhook handling
//! - **Inventory management** with stock tracking and low-stock alerts
//! - **Shipping zones** and rate calculation
//! - **Tax calculation** with region-based rates
//! - **Discount coupons** with percentage, fixed, and free-shipping types
//! - **Product reviews** with moderation and rating statistics
//! - **Hook system** for extensibility (actions and filters)
//!
//! ## Architecture
//!
//! The plugin follows a layered architecture:
//!
//! ```text
//! Handlers (HTTP) -> Services (Business Logic) -> Repositories (Database)
//!                                                      |
//!                                                   Models (Domain)
//! ```
//!
//! ## Quick Start
//!
//! ```ignore
//! use rustcommerce::{RustCommercePlugin, RustCommerceConfig, routes};
//!
//! let config = RustCommerceConfig::new("postgres://localhost/rustcommerce".to_string());
//! let mut plugin = RustCommercePlugin::new(config);
//! plugin.init().await?;
//! plugin.run_migrations().await?;
//! let state = plugin.activate().await?;
//! let router = routes::build_router(state);
//! ```

// ── Module Declarations ────────────────────────────────────────────────

pub mod error;
pub mod config;
pub mod plugin;
pub mod hooks;
pub mod middleware;
pub mod routes;

pub mod models;
pub mod repositories;
pub mod services;
pub mod handlers;

// ── Public Re-exports ──────────────────────────────────────────────────

pub use config::RustCommerceConfig;
pub use error::{Result, RustCommerceError};
pub use plugin::{AppState, RustCommercePlugin};

// ── Constants ──────────────────────────────────────────────────────────

/// Crate version from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Plugin name constant.
pub const PLUGIN_NAME: &str = "RustCommerce";

/// Database table prefix used by all RustCommerce tables.
pub const TABLE_PREFIX: &str = "rc_";
