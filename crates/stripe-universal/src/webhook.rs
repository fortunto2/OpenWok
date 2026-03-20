use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::StripeError;
use crate::types::WebhookEvent;

type HmacSha256 = Hmac<Sha256>;

/// Default tolerance: 5 minutes.
const DEFAULT_TOLERANCE_SECS: u64 = 300;

/// Verify a Stripe webhook signature and parse the event.
///
/// `payload` — raw request body bytes.
/// `signature` — the `Stripe-Signature` header value.
/// `secret` — your webhook signing secret (`whsec_...`).
pub fn verify_and_parse(
    payload: &[u8],
    signature: &str,
    secret: &str,
) -> Result<WebhookEvent, StripeError> {
    verify_signature(payload, signature, secret, DEFAULT_TOLERANCE_SECS)?;
    let event: WebhookEvent = serde_json::from_slice(payload)?;
    Ok(event)
}

/// Verify signature without parsing (if you want to parse separately).
pub fn verify_signature(
    payload: &[u8],
    signature: &str,
    secret: &str,
    tolerance_secs: u64,
) -> Result<(), StripeError> {
    let (timestamp, signatures) = parse_signature_header(signature)?;

    // Check timestamp tolerance
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| StripeError::Http(e.to_string()))?
        .as_secs();

    if now.saturating_sub(timestamp) > tolerance_secs {
        return Err(StripeError::TimestampTooOld);
    }

    // Compute expected signature: HMAC-SHA256(timestamp + "." + payload, secret)
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| StripeError::Http(e.to_string()))?;

    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);

    let expected = hex::encode(mac.finalize().into_bytes());

    // Check if any v1 signature matches
    if signatures
        .iter()
        .any(|sig| constant_time_eq(sig, &expected))
    {
        Ok(())
    } else {
        Err(StripeError::InvalidSignature)
    }
}

/// Parse `Stripe-Signature` header: `t=timestamp,v1=sig1,v1=sig2,...`
fn parse_signature_header(header: &str) -> Result<(u64, Vec<String>), StripeError> {
    let mut timestamp = None;
    let mut signatures = Vec::new();

    for part in header.split(',') {
        let (key, value) = part.split_once('=').ok_or(StripeError::InvalidSignature)?;
        match key.trim() {
            "t" => {
                timestamp = Some(
                    value
                        .trim()
                        .parse::<u64>()
                        .map_err(|_| StripeError::InvalidSignature)?,
                );
            }
            "v1" => {
                signatures.push(value.trim().to_string());
            }
            _ => {} // ignore v0, etc.
        }
    }

    let ts = timestamp.ok_or(StripeError::InvalidSignature)?;
    if signatures.is_empty() {
        return Err(StripeError::InvalidSignature);
    }

    Ok((ts, signatures))
}

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signature(payload: &[u8], secret: &str, timestamp: u64) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);
        let sig = hex::encode(mac.finalize().into_bytes());
        format!("t={timestamp},v1={sig}")
    }

    #[test]
    fn valid_signature_passes() {
        let secret = "whsec_test_secret";
        let payload = br#"{"id":"evt_1","type":"checkout.session.completed","data":{"object":{}}}"#;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let header = make_signature(payload, secret, now);

        let result = verify_and_parse(payload, &header, secret);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().event_type, "checkout.session.completed");
    }

    #[test]
    fn wrong_secret_fails() {
        let payload = br#"{"id":"evt_1","type":"test","data":{"object":{}}}"#;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let header = make_signature(payload, "whsec_real", now);

        let result = verify_and_parse(payload, &header, "whsec_wrong");
        assert!(matches!(result, Err(StripeError::InvalidSignature)));
    }

    #[test]
    fn expired_timestamp_fails() {
        let secret = "whsec_test";
        let payload = br#"{"id":"evt_1","type":"test","data":{"object":{}}}"#;
        let old = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 600; // 10 min ago
        let header = make_signature(payload, secret, old);

        let result = verify_and_parse(payload, &header, secret);
        assert!(matches!(result, Err(StripeError::TimestampTooOld)));
    }

    #[test]
    fn malformed_header_fails() {
        let payload = b"{}";
        let result = verify_signature(payload, "garbage", "secret", 300);
        assert!(matches!(result, Err(StripeError::InvalidSignature)));
    }

    #[test]
    fn missing_v1_fails() {
        let payload = b"{}";
        let result = verify_signature(payload, "t=12345", "secret", 300);
        assert!(matches!(result, Err(StripeError::InvalidSignature)));
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "ab"));
    }
}
