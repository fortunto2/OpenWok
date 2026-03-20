use serde::{Deserialize, Serialize};

// --- Checkout Session ---

/// Parameters to create a Stripe Checkout Session.
#[derive(Debug, Clone, Serialize)]
pub struct CreateCheckoutSessionParams {
    pub mode: CheckoutMode,
    pub success_url: String,
    pub cancel_url: String,
    pub line_items: Vec<LineItem>,
    /// Connect: transfer funds to a connected account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_intent_data: Option<PaymentIntentData>,
    /// Metadata (order_id, etc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckoutMode {
    Payment,
    Subscription,
    Setup,
}

#[derive(Debug, Clone, Serialize)]
pub struct LineItem {
    pub price_data: PriceData,
    pub quantity: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceData {
    pub currency: String,
    pub product_data: ProductData,
    /// Amount in cents.
    pub unit_amount: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductData {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentIntentData {
    pub transfer_data: TransferData,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferData {
    /// Connected account ID (acct_xxx).
    pub destination: String,
    /// Amount in cents to transfer to the connected account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
}

/// Response from Stripe Checkout Session creation.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckoutSession {
    pub id: String,
    pub url: Option<String>,
    pub payment_intent: Option<String>,
    pub payment_status: Option<String>,
    pub status: Option<String>,
    pub amount_total: Option<i64>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

// --- Webhook Events ---

/// Top-level webhook event from Stripe.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: EventData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventData {
    pub object: serde_json::Value,
}

impl WebhookEvent {
    /// Try to parse the event data as a CheckoutSession.
    pub fn as_checkout_session(&self) -> Result<CheckoutSession, serde_json::Error> {
        serde_json::from_value(self.data.object.clone())
    }
}
