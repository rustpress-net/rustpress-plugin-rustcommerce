//! RustCommerce - E-commerce plugin for RustPress

pub mod products;
pub mod cart;
pub mod orders;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init() {
    println!("RustCommerce v{} initialized", VERSION);
}
