//! RustCommerce configuration
//!
//! Plugin configuration with sensible defaults for all settings
//! including database, Stripe, shipping, tax, and inventory.

use serde::{Deserialize, Serialize};

/// Top-level configuration for the RustCommerce plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustCommerceConfig {
    /// Database connection URL (PostgreSQL).
    pub database_url: String,

    /// Maximum number of database connections in the pool.
    #[serde(default = "default_max_connections")]
    pub max_db_connections: u32,

    /// Store configuration.
    #[serde(default)]
    pub store: StoreConfig,

    /// Stripe payment configuration.
    #[serde(default)]
    pub stripe: StripeConfig,

    /// Inventory management configuration.
    #[serde(default)]
    pub inventory: InventoryConfig,

    /// Tax configuration.
    #[serde(default)]
    pub tax: TaxConfig,

    /// Shipping configuration.
    #[serde(default)]
    pub shipping: ShippingConfig,

    /// Checkout configuration.
    #[serde(default)]
    pub checkout: CheckoutConfig,

    /// Review/rating configuration.
    #[serde(default)]
    pub reviews: ReviewConfig,
}

/// General store settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    /// Store name for display purposes.
    #[serde(default = "default_store_name")]
    pub name: String,

    /// Default currency code (ISO 4217).
    #[serde(default = "default_currency")]
    pub currency: String,

    /// Whether prices include tax.
    #[serde(default)]
    pub prices_include_tax: bool,

    /// Number of products per page in listings.
    #[serde(default = "default_page_size")]
    pub products_per_page: u32,

    /// Number of orders per page in listings.
    #[serde(default = "default_page_size")]
    pub orders_per_page: u32,
}

/// Stripe payment gateway settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeConfig {
    /// Stripe secret key (sk_live_... or sk_test_...).
    #[serde(default)]
    pub secret_key: String,

    /// Stripe publishable key (pk_live_... or pk_test_...).
    #[serde(default)]
    pub publishable_key: String,

    /// Stripe webhook signing secret (whsec_...).
    #[serde(default)]
    pub webhook_secret: String,

    /// Whether Stripe is enabled for payment processing.
    #[serde(default)]
    pub enabled: bool,
}

/// Inventory tracking settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryConfig {
    /// Whether to track inventory globally.
    #[serde(default = "default_true")]
    pub track_inventory: bool,

    /// Default low stock threshold (sends alerts when reached).
    #[serde(default = "default_low_stock_threshold")]
    pub low_stock_threshold: i32,

    /// Whether to allow backorders by default.
    #[serde(default)]
    pub allow_backorders: bool,

    /// Whether to reserve stock when items are added to cart.
    #[serde(default)]
    pub reserve_stock_on_cart: bool,

    /// Minutes to hold reserved stock before releasing.
    #[serde(default = "default_stock_reserve_minutes")]
    pub stock_reserve_minutes: u32,
}

/// Tax calculation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxConfig {
    /// Whether tax calculation is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether to calculate tax on shipping.
    #[serde(default)]
    pub tax_shipping: bool,

    /// Tax rounding precision (decimal places).
    #[serde(default = "default_tax_precision")]
    pub rounding_precision: u32,
}

/// Shipping settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingConfig {
    /// Whether shipping calculation is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Free shipping threshold amount (None = no free shipping).
    pub free_shipping_threshold: Option<rust_decimal::Decimal>,

    /// Default shipping country code.
    #[serde(default = "default_country")]
    pub default_country: String,
}

/// Checkout workflow settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutConfig {
    /// Whether guest checkout (no account required) is allowed.
    #[serde(default = "default_true")]
    pub allow_guest_checkout: bool,

    /// Whether to require phone number at checkout.
    #[serde(default)]
    pub require_phone: bool,

    /// Whether to require shipping address (false for digital-only stores).
    #[serde(default = "default_true")]
    pub require_shipping_address: bool,

    /// Minimum order amount to proceed with checkout.
    pub minimum_order_amount: Option<rust_decimal::Decimal>,

    /// Cart expiry in minutes (None = no expiry).
    #[serde(default = "default_cart_expiry")]
    pub cart_expiry_minutes: Option<u32>,
}

/// Review/rating settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    /// Whether reviews are enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether reviews require manual approval.
    #[serde(default = "default_true")]
    pub require_approval: bool,

    /// Whether only verified purchasers can leave reviews.
    #[serde(default)]
    pub verified_purchases_only: bool,

    /// Minimum rating value.
    #[serde(default = "default_min_rating")]
    pub min_rating: u8,

    /// Maximum rating value.
    #[serde(default = "default_max_rating")]
    pub max_rating: u8,
}

// ── Default Functions ────────────────────────────────────────────────────

fn default_max_connections() -> u32 {
    10
}

fn default_store_name() -> String {
    "RustCommerce Store".to_string()
}

fn default_currency() -> String {
    "USD".to_string()
}

fn default_page_size() -> u32 {
    20
}

fn default_true() -> bool {
    true
}

fn default_low_stock_threshold() -> i32 {
    5
}

fn default_stock_reserve_minutes() -> u32 {
    30
}

fn default_tax_precision() -> u32 {
    2
}

fn default_country() -> String {
    "US".to_string()
}

fn default_cart_expiry() -> Option<u32> {
    Some(1440) // 24 hours
}

fn default_min_rating() -> u8 {
    1
}

fn default_max_rating() -> u8 {
    5
}

// ── Default Implementations ─────────────────────────────────────────────

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            name: default_store_name(),
            currency: default_currency(),
            prices_include_tax: false,
            products_per_page: default_page_size(),
            orders_per_page: default_page_size(),
        }
    }
}

impl Default for StripeConfig {
    fn default() -> Self {
        Self {
            secret_key: String::new(),
            publishable_key: String::new(),
            webhook_secret: String::new(),
            enabled: false,
        }
    }
}

impl Default for InventoryConfig {
    fn default() -> Self {
        Self {
            track_inventory: true,
            low_stock_threshold: default_low_stock_threshold(),
            allow_backorders: false,
            reserve_stock_on_cart: false,
            stock_reserve_minutes: default_stock_reserve_minutes(),
        }
    }
}

impl Default for TaxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tax_shipping: false,
            rounding_precision: default_tax_precision(),
        }
    }
}

impl Default for ShippingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            free_shipping_threshold: None,
            default_country: default_country(),
        }
    }
}

impl Default for CheckoutConfig {
    fn default() -> Self {
        Self {
            allow_guest_checkout: true,
            require_phone: false,
            require_shipping_address: true,
            minimum_order_amount: None,
            cart_expiry_minutes: default_cart_expiry(),
        }
    }
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_approval: true,
            verified_purchases_only: false,
            min_rating: default_min_rating(),
            max_rating: default_max_rating(),
        }
    }
}

impl RustCommerceConfig {
    /// Create a new configuration with sensible defaults.
    /// Only `database_url` is required.
    pub fn new(database_url: String) -> Self {
        Self {
            database_url,
            max_db_connections: default_max_connections(),
            store: StoreConfig::default(),
            stripe: StripeConfig::default(),
            inventory: InventoryConfig::default(),
            tax: TaxConfig::default(),
            shipping: ShippingConfig::default(),
            checkout: CheckoutConfig::default(),
            reviews: ReviewConfig::default(),
        }
    }

    /// Validate the configuration and return errors if invalid.
    pub fn validate(&self) -> std::result::Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("database_url is required".to_string());
        }

        if self.stripe.enabled {
            if self.stripe.secret_key.is_empty() {
                errors.push("stripe.secret_key is required when Stripe is enabled".to_string());
            }
            if self.stripe.webhook_secret.is_empty() {
                errors.push(
                    "stripe.webhook_secret is required when Stripe is enabled".to_string(),
                );
            }
        }

        if self.max_db_connections == 0 {
            errors.push("max_db_connections must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
