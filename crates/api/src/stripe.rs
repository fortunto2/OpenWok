use openwok_core::pricing::PricingBreakdown;
use rust_decimal::prelude::ToPrimitive;
use stripe_universal::types::{
    CheckoutMode, CreateCheckoutSessionParams, LineItem, PaymentIntentData, PriceData, ProductData,
    TransferData,
};

/// Convert Money amount to Stripe cents (integer).
fn to_cents(money: &openwok_core::money::Money) -> i64 {
    (money.amount() * rust_decimal::Decimal::from(100))
        .round()
        .to_i64()
        .unwrap_or(0)
}

/// Build Stripe Checkout Session params from an order's pricing breakdown.
///
/// - `order_id`: used in metadata for webhook reconciliation
/// - `success_url` / `cancel_url`: redirect URLs after checkout
/// - `restaurant_account_id`: Stripe Connect account for the restaurant (if available)
pub fn build_checkout_params(
    pricing: &PricingBreakdown,
    order_id: &str,
    success_url: &str,
    cancel_url: &str,
    restaurant_account_id: Option<&str>,
) -> CreateCheckoutSessionParams {
    let total_cents = to_cents(&pricing.total());

    // Single line item for the total order amount
    let line_items = vec![LineItem {
        price_data: PriceData {
            currency: "usd".into(),
            product_data: ProductData {
                name: "OpenWok Order".into(),
            },
            unit_amount: total_cents,
        },
        quantity: 1,
    }];

    // If restaurant has a connected account, transfer their share
    let payment_intent_data = restaurant_account_id.map(|acct_id| {
        let restaurant_cents = to_cents(&pricing.food_total);
        PaymentIntentData {
            transfer_data: TransferData {
                destination: acct_id.to_string(),
                amount: Some(restaurant_cents),
            },
        }
    });

    let metadata = Some(
        [("order_id".to_string(), order_id.to_string())]
            .into_iter()
            .collect(),
    );

    CreateCheckoutSessionParams {
        mode: CheckoutMode::Payment,
        success_url: success_url.to_string(),
        cancel_url: cancel_url.to_string(),
        line_items,
        payment_intent_data,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openwok_core::money::Money;
    use openwok_core::pricing::PricingBreakdown;

    fn test_pricing() -> PricingBreakdown {
        PricingBreakdown {
            food_total: Money::from("25.00"),
            delivery_fee: Money::from("5.00"),
            tip: Money::from("3.00"),
            federal_fee: Money::from("1.00"),
            local_ops_fee: Money::from("2.50"),
            processing_fee: Money::from("1.36"),
        }
    }

    #[test]
    fn to_cents_converts_correctly() {
        assert_eq!(to_cents(&Money::from("25.00")), 2500);
        assert_eq!(to_cents(&Money::from("1.36")), 136);
        assert_eq!(to_cents(&Money::from("0.30")), 30);
    }

    #[test]
    fn build_params_without_connect() {
        let pricing = test_pricing();
        let params = build_checkout_params(
            &pricing,
            "ord_123",
            "https://example.com/success",
            "https://example.com/cancel",
            None,
        );

        assert_eq!(params.line_items.len(), 1);
        // Total: 25+5+3+1+2.5+1.36 = 37.86 → 3786 cents
        assert_eq!(params.line_items[0].price_data.unit_amount, 3786);
        assert!(params.payment_intent_data.is_none());
        assert_eq!(
            params.metadata.as_ref().unwrap().get("order_id").unwrap(),
            "ord_123"
        );
    }

    #[test]
    fn build_params_with_connect() {
        let pricing = test_pricing();
        let params = build_checkout_params(
            &pricing,
            "ord_456",
            "https://example.com/ok",
            "https://example.com/no",
            Some("acct_restaurant_abc"),
        );

        let pid = params.payment_intent_data.unwrap();
        assert_eq!(pid.transfer_data.destination, "acct_restaurant_abc");
        assert_eq!(pid.transfer_data.amount, Some(2500)); // food_total = $25
    }
}
