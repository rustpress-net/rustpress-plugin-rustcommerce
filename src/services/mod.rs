//! RustCommerce business logic services
//!
//! Services encapsulate domain logic, calling repositories for data access
//! and coordinating cross-cutting concerns (hooks, validation, etc.).

pub mod product_service;
pub mod cart_service;
pub mod checkout_service;
pub mod order_service;
pub mod payment_service;
pub mod inventory_service;
pub mod shipping_service;
pub mod tax_service;
pub mod coupon_service;
pub mod review_service;

pub use product_service::ProductService;
pub use cart_service::CartService;
pub use checkout_service::CheckoutService;
pub use order_service::OrderService;
pub use payment_service::PaymentService;
pub use inventory_service::InventoryService;
pub use shipping_service::ShippingService;
pub use tax_service::TaxService;
pub use coupon_service::CouponService;
pub use review_service::ReviewService;
