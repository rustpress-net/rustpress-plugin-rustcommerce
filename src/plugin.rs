//! RustCommerce plugin lifecycle
//!
//! Plugin initialization, activation, deactivation, and shared application state.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::RustCommerceConfig;
use crate::error::{Result, RustCommerceError};

/// Shared application state available to all handlers via Axum's `State` extractor.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<RustCommerceConfig>,
}

impl AppState {
    /// Dereference config helper.
    pub fn config(&self) -> &RustCommerceConfig {
        &self.config
    }
}

// Implement Deref so handlers can access config fields conveniently
impl std::ops::Deref for AppState {
    type Target = RustCommerceConfig;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

/// The main plugin struct managing lifecycle.
pub struct RustCommercePlugin {
    config: RustCommerceConfig,
    pool: Option<PgPool>,
}

impl RustCommercePlugin {
    /// Create a new plugin instance with the given configuration.
    pub fn new(config: RustCommerceConfig) -> Self {
        Self { config, pool: None }
    }

    /// Initialize the plugin: validate config, create database pool, run migrations.
    pub async fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing RustCommerce plugin v{}", crate::VERSION);

        // Validate configuration
        self.config.validate().map_err(|errors| {
            RustCommerceError::Configuration(format!(
                "Configuration validation failed: {}",
                errors.join(", ")
            ))
        })?;

        // Create database connection pool
        let pool = PgPoolOptions::new()
            .max_connections(self.config.max_db_connections)
            .connect(&self.config.database_url)
            .await
            .map_err(|e| {
                RustCommerceError::Configuration(format!("Database connection failed: {}", e))
            })?;

        tracing::info!(
            max_connections = self.config.max_db_connections,
            "Database pool created"
        );

        self.pool = Some(pool);

        Ok(())
    }

    /// Activate the plugin: register hooks, set up routes.
    pub async fn activate(&self) -> Result<AppState> {
        let pool = self
            .pool
            .clone()
            .ok_or_else(|| RustCommerceError::Internal("Plugin not initialized".to_string()))?;

        tracing::info!("Activating RustCommerce plugin");

        // Register default hooks
        self.register_default_hooks().await;

        let state = AppState {
            pool,
            config: Arc::new(self.config.clone()),
        };

        tracing::info!("RustCommerce plugin activated successfully");

        Ok(state)
    }

    /// Deactivate the plugin: clean up resources.
    pub async fn deactivate(&mut self) -> Result<()> {
        tracing::info!("Deactivating RustCommerce plugin");

        // Clear hooks
        crate::hooks::HookRegistry::clear_all().await;

        // Close database pool
        if let Some(pool) = self.pool.take() {
            pool.close().await;
            tracing::info!("Database pool closed");
        }

        tracing::info!("RustCommerce plugin deactivated");
        Ok(())
    }

    /// Run database migrations.
    pub async fn run_migrations(&self) -> Result<()> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| RustCommerceError::Internal("Plugin not initialized".to_string()))?;

        tracing::info!("Running RustCommerce database migrations");

        // In production, use sqlx::migrate! macro with the migrations directory.
        // For standalone builds, migrations can be run manually.
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .map_err(|e| RustCommerceError::Migration(e.to_string()))?;

        tracing::info!("Database migrations completed");
        Ok(())
    }

    /// Get a reference to the database pool.
    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    /// Register default action hooks for logging and side effects.
    async fn register_default_hooks(&self) {
        // Log order creation
        crate::hooks::HookRegistry::register_action("order_created", |data| async move {
            if let Some(order_number) = data.get("order_number").and_then(|v| v.as_str()) {
                tracing::info!(order_number = order_number, "Hook: order_created fired");
            }
        })
        .await;

        // Log stock alerts
        crate::hooks::HookRegistry::register_action("stock_low", |data| async move {
            if let Some(product_name) = data.get("product_name").and_then(|v| v.as_str()) {
                tracing::warn!(
                    product = product_name,
                    "Hook: stock_low fired - reorder recommended"
                );
            }
        })
        .await;

        // Log payment events
        crate::hooks::HookRegistry::register_action("payment_completed", |data| async move {
            if let Some(order_id) = data.get("order_id").and_then(|v| v.as_str()) {
                tracing::info!(order_id = order_id, "Hook: payment_completed fired");
            }
        })
        .await;

        crate::hooks::HookRegistry::register_action("payment_failed", |data| async move {
            if let Some(order_id) = data.get("order_id").and_then(|v| v.as_str()) {
                tracing::error!(order_id = order_id, "Hook: payment_failed fired");
            }
        })
        .await;

        tracing::debug!("Default hooks registered");
    }
}
