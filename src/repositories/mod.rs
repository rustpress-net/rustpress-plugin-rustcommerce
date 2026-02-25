//! RustCommerce data repositories
//!
//! Each repository encapsulates direct database access for a domain entity.
//! All methods accept `&sqlx::PgPool` for connection management.

pub mod product_repo;
pub mod category_repo;
pub mod cart_repo;
pub mod order_repo;
pub mod customer_repo;
pub mod payment_repo;
pub mod coupon_repo;
pub mod review_repo;

pub use product_repo::ProductRepository;
pub use category_repo::CategoryRepository;
pub use cart_repo::CartRepository;
pub use order_repo::OrderRepository;
pub use customer_repo::CustomerRepository;
pub use payment_repo::PaymentRepository;
pub use coupon_repo::CouponRepository;
pub use review_repo::ReviewRepository;
