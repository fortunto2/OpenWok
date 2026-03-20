#[cfg(all(feature = "reqwest-backend", feature = "worker-backend"))]
compile_error!("Features `reqwest-backend` and `worker-backend` are mutually exclusive");

use crate::error::{StripeApiError, StripeError};
use crate::types::{CheckoutSession, CreateCheckoutSessionParams};

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

/// Stripe API client. Works with reqwest (native) or worker::Fetch (wasm32).
pub struct StripeClient {
    secret_key: String,
    #[cfg(feature = "reqwest-backend")]
    http: reqwest::Client,
}

impl StripeClient {
    /// Create a new Stripe client.
    #[cfg(feature = "reqwest-backend")]
    pub fn new(secret_key: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Create a new Stripe client (worker backend — no HTTP client needed, uses global Fetch).
    #[cfg(all(feature = "worker-backend", not(feature = "reqwest-backend")))]
    pub fn new(secret_key: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
        }
    }

    /// Create a Stripe Checkout Session. Returns the session with a redirect URL.
    #[cfg(feature = "reqwest-backend")]
    pub async fn create_checkout_session(
        &self,
        params: &CreateCheckoutSessionParams,
    ) -> Result<CheckoutSession, StripeError> {
        let body = serialize_to_form(params)?;

        let resp = self
            .http
            .post(format!("{STRIPE_API_BASE}/checkout/sessions"))
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| StripeError::Http(e.to_string()))?;

        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| StripeError::Http(e.to_string()))?;

        if status.is_success() {
            Ok(serde_json::from_slice(&bytes)?)
        } else {
            let api_err: Result<ApiErrorWrapper, _> = serde_json::from_slice(&bytes);
            match api_err {
                Ok(wrapper) => Err(StripeError::Api(wrapper.error.message)),
                Err(_) => Err(StripeError::Http(format!("HTTP {status}"))),
            }
        }
    }

    /// Create a Stripe Checkout Session via worker::Fetch.
    #[cfg(all(feature = "worker-backend", not(feature = "reqwest-backend")))]
    pub async fn create_checkout_session(
        &self,
        params: &CreateCheckoutSessionParams,
    ) -> Result<CheckoutSession, StripeError> {
        use worker::{Fetch, Headers, Method, Request, RequestInit};

        let body = serialize_to_form(params)?;

        let mut headers = Headers::new();
        headers
            .set("Authorization", &format!("Bearer {}", self.secret_key))
            .map_err(|e| StripeError::Http(e.to_string()))?;
        headers
            .set("Content-Type", "application/x-www-form-urlencoded")
            .map_err(|e| StripeError::Http(e.to_string()))?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(body.into()));

        let req = Request::new_with_init(&format!("{STRIPE_API_BASE}/checkout/sessions"), &init)
            .map_err(|e| StripeError::Http(e.to_string()))?;

        let mut resp = Fetch::Request(req)
            .send()
            .await
            .map_err(|e| StripeError::Http(e.to_string()))?;

        let text = resp
            .text()
            .await
            .map_err(|e| StripeError::Http(e.to_string()))?;

        let status = resp.status_code();
        if (200..300).contains(&status) {
            Ok(serde_json::from_str(&text)?)
        } else {
            let api_err: Result<ApiErrorWrapper, _> = serde_json::from_str(&text);
            match api_err {
                Ok(wrapper) => Err(StripeError::Api(wrapper.error.message)),
                Err(_) => Err(StripeError::Http(format!("HTTP {status}"))),
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct ApiErrorWrapper {
    error: StripeApiError,
}

/// Serialize checkout params to Stripe's form-encoded format.
///
/// Stripe expects nested params as `key[nested]=value`, e.g.:
/// `line_items[0][price_data][unit_amount]=2500`
fn serialize_to_form(params: &CreateCheckoutSessionParams) -> Result<String, StripeError> {
    let mut parts: Vec<(String, String)> = Vec::new();

    // mode
    parts.push((
        "mode".into(),
        serde_json::to_value(&params.mode)
            .map_err(|e| StripeError::Http(e.to_string()))?
            .as_str()
            .unwrap_or("payment")
            .to_string(),
    ));

    parts.push(("success_url".into(), params.success_url.clone()));
    parts.push(("cancel_url".into(), params.cancel_url.clone()));

    // line_items
    for (i, item) in params.line_items.iter().enumerate() {
        let prefix = format!("line_items[{i}]");
        parts.push((format!("{prefix}[quantity]"), item.quantity.to_string()));
        parts.push((
            format!("{prefix}[price_data][currency]"),
            item.price_data.currency.clone(),
        ));
        parts.push((
            format!("{prefix}[price_data][unit_amount]"),
            item.price_data.unit_amount.to_string(),
        ));
        parts.push((
            format!("{prefix}[price_data][product_data][name]"),
            item.price_data.product_data.name.clone(),
        ));
    }

    // payment_intent_data.transfer_data (Connect)
    if let Some(ref pid) = params.payment_intent_data {
        parts.push((
            "payment_intent_data[transfer_data][destination]".into(),
            pid.transfer_data.destination.clone(),
        ));
        if let Some(amount) = pid.transfer_data.amount {
            parts.push((
                "payment_intent_data[transfer_data][amount]".into(),
                amount.to_string(),
            ));
        }
    }

    // metadata
    if let Some(ref meta) = params.metadata {
        for (k, v) in meta {
            parts.push((format!("metadata[{k}]"), v.clone()));
        }
    }

    Ok(parts
        .iter()
        .map(|(k, v)| format!("{}={}", urlencod(k), urlencod(v)))
        .collect::<Vec<_>>()
        .join("&"))
}

/// Minimal URL encoding for form values.
fn urlencod(s: &str) -> String {
    s.replace('%', "%25")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('+', "%2B")
        .replace(' ', "+")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn serialize_basic_checkout() {
        let params = CreateCheckoutSessionParams {
            mode: CheckoutMode::Payment,
            success_url: "https://example.com/success".into(),
            cancel_url: "https://example.com/cancel".into(),
            line_items: vec![LineItem {
                price_data: PriceData {
                    currency: "usd".into(),
                    product_data: ProductData {
                        name: "Pad Thai".into(),
                    },
                    unit_amount: 1299,
                },
                quantity: 1,
            }],
            payment_intent_data: None,
            metadata: None,
        };

        let form = serialize_to_form(&params).unwrap();
        assert!(form.contains("mode=payment"));
        assert!(form.contains("line_items[0][price_data][unit_amount]=1299"));
        assert!(form.contains("line_items[0][price_data][currency]=usd"));
        assert!(form.contains("line_items[0][price_data][product_data][name]=Pad+Thai"));
        assert!(form.contains("line_items[0][quantity]=1"));
    }

    #[test]
    fn serialize_with_connect_transfer() {
        let params = CreateCheckoutSessionParams {
            mode: CheckoutMode::Payment,
            success_url: "https://example.com/ok".into(),
            cancel_url: "https://example.com/no".into(),
            line_items: vec![LineItem {
                price_data: PriceData {
                    currency: "usd".into(),
                    product_data: ProductData {
                        name: "Order".into(),
                    },
                    unit_amount: 3786,
                },
                quantity: 1,
            }],
            payment_intent_data: Some(PaymentIntentData {
                transfer_data: TransferData {
                    destination: "acct_restaurant_123".into(),
                    amount: Some(2500),
                },
            }),
            metadata: Some([("order_id".to_string(), "ord_abc".to_string())].into()),
        };

        let form = serialize_to_form(&params).unwrap();
        assert!(
            form.contains("payment_intent_data[transfer_data][destination]=acct_restaurant_123")
        );
        assert!(form.contains("payment_intent_data[transfer_data][amount]=2500"));
        assert!(form.contains("metadata[order_id]=ord_abc"));
    }
}
