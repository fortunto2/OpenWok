use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(value_type = String, example = "12.99")]
pub struct Money(Decimal);

impl Money {
    pub fn new(amount: Decimal) -> Self {
        Self(amount)
    }

    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    pub fn amount(&self) -> Decimal {
        self.0
    }

    /// Round to 2 decimal places (cents).
    pub fn round_cents(self) -> Self {
        Self(self.0.round_dp(2))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rounded = self.0.round_dp(2);
        write!(f, "${rounded:.2}")
    }
}

impl From<&str> for Money {
    fn from(s: &str) -> Self {
        let s = s.trim().trim_start_matches('$');
        Self(Decimal::from_str(s).expect("invalid money string"))
    }
}

impl From<Decimal> for Money {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul<Decimal> for Money {
    type Output = Self;
    fn mul(self, rhs: Decimal) -> Self {
        Self(self.0 * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_as_dollars() {
        let m = Money::from("25.50");
        assert_eq!(m.to_string(), "$25.50");
    }

    #[test]
    fn display_pads_cents() {
        let m = Money::from("5");
        assert_eq!(m.to_string(), "$5.00");
    }

    #[test]
    fn from_str_strips_dollar_sign() {
        let m = Money::from("$12.99");
        assert_eq!(m.amount(), Decimal::from_str("12.99").unwrap());
    }

    #[test]
    fn addition() {
        let a = Money::from("10.00");
        let b = Money::from("5.50");
        assert_eq!((a + b).to_string(), "$15.50");
    }

    #[test]
    fn subtraction() {
        let a = Money::from("10.00");
        let b = Money::from("3.25");
        assert_eq!((a - b).to_string(), "$6.75");
    }

    #[test]
    fn multiplication_by_decimal() {
        let m = Money::from("100.00");
        let rate = Decimal::from_str("0.029").unwrap();
        let result = (m * rate).round_cents();
        assert_eq!(result.to_string(), "$2.90");
    }

    #[test]
    fn zero() {
        assert_eq!(Money::zero().to_string(), "$0.00");
    }

    #[test]
    fn ordering() {
        let a = Money::from("5.00");
        let b = Money::from("10.00");
        assert!(a < b);
    }

    #[test]
    fn serde_roundtrip() {
        let m = Money::from("42.99");
        let json = serde_json::to_string(&m).unwrap();
        let back: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
