//! Checkout service
//!
//! Orchestrates the cart-to-order-to-payment workflow.

use sqlx::PgPool;
use validator::Validate;

use crate::config::RustCommerceConfig;
use crate::error::{Result, RustCommerceError};
use crate::hooks::HookRegistry;
use crate::models::order::*;
use crate::models::payment::{CreatePaymentRequest, PaymentMethod};
use crate::repositories::{CartRepository, CouponRepository, CustomerRepository, OrderRepository, ProductRepository};
use crate::services::{PaymentService, TaxService, ShippingService};

/// Service for orchestrating the checkout flow.
pub struct CheckoutService;

impl CheckoutService {
    /// Process a checkout: validate cart, create order, create payment, clean up cart.
    pub async fn process(
        pool: &PgPool,
        config: &RustCommerceConfig,
        req: &CheckoutRequest,
    ) -> Result<OrderWithItems> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        // 1. Load and validate cart
        let cart = CartRepository::find_by_id(pool, req.cart_id).await?;
        let cart_items = CartRepository::list_items(pool, req.cart_id).await?;

        if cart_items.is_empty() {
            return Err(RustCommerceError::EmptyCart);
        }

        // 2. Verify stock for all items
        for item in &cart_items {
            let product = ProductRepository::find_by_id(pool, item.product_id).await?;
            if product.track_inventory && !product.allow_backorders {
                if product.stock_quantity < item.quantity {
                    return Err(RustCommerceError::InsufficientStock {
                        product_id: item.product_id,
                        requested: item.quantity,
                        available: product.stock_quantity,
                    });
                }
            }
        }

        // 3. Calculate tax
        let subtotal = cart.subtotal;
        let tax_total = TaxService::calculate_tax_for_address(
            pool,
            subtotal,
            &req.shipping_address.country,
            req.shipping_address.state.as_deref(),
        )
        .await?;

        // 4. Calculate shipping
        let shipping_total = ShippingService::calculate_for_order(
            pool,
            &req.shipping_address.country,
            req.shipping_address.state.as_deref(),
            subtotal,
            cart_items.len() as i32,
        )
        .await?;

        // 5. Calculate discount
        let discount_total = cart.discount_total;

        // 6. Check minimum order amount
        let grand_total = subtotal + tax_total + shipping_total - discount_total;
        if let Some(min) = config.checkout.minimum_order_amount {
            if grand_total < min {
                return Err(RustCommerceError::MinimumOrderNotMet {
                    required: min,
                    current: grand_total,
                });
            }
        }

        // 7. Generate order number and create order
        let order_number = generate_order_number();
        let coupon_code = cart.coupon_code.as_deref();

        let order = OrderRepository::create(
            pool,
            req,
            &order_number,
            subtotal,
            tax_total,
            shipping_total,
            discount_total,
            grand_total,
            coupon_code,
        )
        .await?;

        // 8. Create order items and deduct stock
        let mut order_items = Vec::new();
        for item in &cart_items {
            let product = ProductRepository::find_by_id(pool, item.product_id).await?;

            let order_item = OrderRepository::create_item(
                pool,
                order.id,
                item.product_id,
                item.variant_id,
                &product.name,
                None,
                product.sku.as_deref(),
                item.quantity,
                item.unit_price,
                item.total_price,
                rust_decimal::Decimal::ZERO, // Individual item tax handled at order level
                rust_decimal::Decimal::ZERO,
            )
            .await?;

            // Deduct inventory
            if product.track_inventory {
                ProductRepository::update_stock(pool, item.product_id, -item.quantity).await?;
            }

            order_items.push(order_item);
        }

        // 9. Increment coupon usage if applicable
        if let Some(ref code) = cart.coupon_code {
            if let Ok(coupon) = CouponRepository::find_by_code(pool, code).await {
                CouponRepository::increment_usage(pool, coupon.id).await?;
            }
        }

        // 10. Update customer stats if registered
        if let Some(customer_id) = req.customer_id {
            let _ = CustomerRepository::record_order(pool, customer_id, grand_total).await;
        }

        // 11. Create payment record
        let payment_method_str = req.payment_method.as_deref().unwrap_or("stripe");
        let payment_method = match payment_method_str {
            "stripe" => PaymentMethod::Stripe,
            "paypal" => PaymentMethod::Paypal,
            "bank_transfer" => PaymentMethod::BankTransfer,
            "cash_on_delivery" => PaymentMethod::CashOnDelivery,
            _ => PaymentMethod::Manual,
        };

        let _payment = PaymentService::create_payment(
            pool,
            &CreatePaymentRequest {
                order_id: order.id,
                payment_method,
                amount: grand_total,
                currency: Some(order.currency.clone()),
                metadata: None,
            },
        )
        .await?;

        // 12. Delete the cart
        CartRepository::delete(pool, req.cart_id).await?;

        // 13. Fire hooks
        HookRegistry::fire_action("order_created", &serde_json::to_value(&order)?).await;

        tracing::info!(
            order_id = %order.id,
            order_number = %order.order_number,
            grand_total = %grand_total,
            "Checkout completed"
        );

        Ok(OrderWithItems {
            order,
            items: order_items,
        })
    }
}
