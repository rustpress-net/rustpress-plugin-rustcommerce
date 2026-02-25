//! Hook registration and firing system
//!
//! Provides a simple hook system for RustCommerce actions and filters.
//! Actions are fire-and-forget notifications; filters transform values.
//!
//! In a full RustPress integration, these would delegate to the core
//! hook system. This standalone implementation uses a global registry.

use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type alias for action callback functions.
pub type ActionCallback = Arc<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
>;

/// Type alias for filter callback functions.
pub type FilterCallback = Arc<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = Value> + Send>> + Send + Sync,
>;

/// Global hook registry using lazy initialization.
static HOOK_REGISTRY: std::sync::LazyLock<RwLock<HookRegistryInner>> =
    std::sync::LazyLock::new(|| RwLock::new(HookRegistryInner::new()));

/// Internal registry holding all registered hooks.
struct HookRegistryInner {
    actions: HashMap<String, Vec<ActionCallback>>,
    filters: HashMap<String, Vec<FilterCallback>>,
}

impl HookRegistryInner {
    fn new() -> Self {
        Self {
            actions: HashMap::new(),
            filters: HashMap::new(),
        }
    }
}

/// Public API for the hook system.
pub struct HookRegistry;

impl HookRegistry {
    /// Register an action hook callback.
    ///
    /// Actions are one-way notifications fired when events occur.
    /// Multiple callbacks can be registered for the same hook name.
    pub async fn register_action<F, Fut>(name: &str, callback: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let callback = Arc::new(move |v: Value| -> Pin<Box<dyn Future<Output = ()> + Send>> {
            Box::pin(callback(v))
        });

        let mut registry = HOOK_REGISTRY.write().await;
        registry
            .actions
            .entry(name.to_string())
            .or_default()
            .push(callback);

        tracing::debug!(hook = name, "Action hook registered");
    }

    /// Register a filter hook callback.
    ///
    /// Filters receive a value and return a (potentially modified) value.
    /// Multiple filters for the same hook are applied in registration order.
    pub async fn register_filter<F, Fut>(name: &str, callback: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Value> + Send + 'static,
    {
        let callback = Arc::new(move |v: Value| -> Pin<Box<dyn Future<Output = Value> + Send>> {
            Box::pin(callback(v))
        });

        let mut registry = HOOK_REGISTRY.write().await;
        registry
            .filters
            .entry(name.to_string())
            .or_default()
            .push(callback);

        tracing::debug!(hook = name, "Filter hook registered");
    }

    /// Fire an action hook, notifying all registered callbacks.
    ///
    /// Callbacks are executed concurrently. Errors in callbacks are
    /// logged but do not propagate (fire-and-forget semantics).
    pub async fn fire_action(name: &str, data: &Value) {
        let registry = HOOK_REGISTRY.read().await;

        if let Some(callbacks) = registry.actions.get(name) {
            tracing::debug!(
                hook = name,
                callback_count = callbacks.len(),
                "Firing action hook"
            );

            let mut handles = Vec::new();
            for callback in callbacks {
                let cb = callback.clone();
                let data = data.clone();
                handles.push(tokio::spawn(async move {
                    cb(data).await;
                }));
            }

            for handle in handles {
                if let Err(e) = handle.await {
                    tracing::error!(
                        hook = name,
                        error = %e,
                        "Action hook callback panicked"
                    );
                }
            }
        }
    }

    /// Apply a filter hook, passing a value through all registered callbacks.
    ///
    /// Filters are applied sequentially in registration order.
    /// The final transformed value is returned.
    pub async fn apply_filter(name: &str, mut value: Value) -> Value {
        let registry = HOOK_REGISTRY.read().await;

        if let Some(callbacks) = registry.filters.get(name) {
            tracing::debug!(
                hook = name,
                callback_count = callbacks.len(),
                "Applying filter hook"
            );

            for callback in callbacks {
                value = callback(value).await;
            }
        }

        value
    }

    /// Check how many callbacks are registered for a given action.
    pub async fn action_count(name: &str) -> usize {
        let registry = HOOK_REGISTRY.read().await;
        registry
            .actions
            .get(name)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Check how many callbacks are registered for a given filter.
    pub async fn filter_count(name: &str) -> usize {
        let registry = HOOK_REGISTRY.read().await;
        registry
            .filters
            .get(name)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Clear all registered hooks (primarily for testing).
    pub async fn clear_all() {
        let mut registry = HOOK_REGISTRY.write().await;
        registry.actions.clear();
        registry.filters.clear();
        tracing::debug!("All hooks cleared");
    }
}
