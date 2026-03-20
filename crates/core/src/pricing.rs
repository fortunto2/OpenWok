use crate::money::Money;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The 6-line open-book receipt — zero hidden markup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct PricingBreakdown {
    pub food_total: Money,
    pub delivery_fee: Money,
    pub tip: Money,
    pub federal_fee: Money,
    pub local_ops_fee: Money,
    pub processing_fee: Money,
}

impl PricingBreakdown {
    pub fn total(&self) -> Money {
        self.food_total
            + self.delivery_fee
            + self.tip
            + self.federal_fee
            + self.local_ops_fee
            + self.processing_fee
    }
}

/// Calculate the open-book pricing breakdown.
///
/// - `federal_fee`: always $1.00
/// - `processing_fee`: (food + delivery + tip + federal + local_ops) * 2.9% + $0.30
pub fn calculate_pricing(
    food_total: Money,
    delivery_fee: Money,
    tip: Money,
    local_ops_fee: Money,
) -> PricingBreakdown {
    let federal_fee = Money::from("1.00");
    let subtotal = food_total + delivery_fee + tip + federal_fee + local_ops_fee;
    let rate = Decimal::from_str("0.029").unwrap();
    let processing_fee = (subtotal * rate + Money::from("0.30")).round_cents();

    PricingBreakdown {
        food_total,
        delivery_fee,
        tip,
        federal_fee,
        local_ops_fee,
        processing_fee,
    }
}

impl fmt::Display for PricingBreakdown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Food Total:     {}", self.food_total)?;
        writeln!(f, "Delivery Fee:   {}", self.delivery_fee)?;
        writeln!(f, "Tip:            {}", self.tip)?;
        writeln!(f, "Federal Fee:    {}", self.federal_fee)?;
        writeln!(f, "Local Ops Fee:  {}", self.local_ops_fee)?;
        writeln!(f, "Processing Fee: {}", self.processing_fee)?;
        write!(f, "Total:          {}", self.total())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_sums_all_fields() {
        let b = PricingBreakdown {
            food_total: Money::from("25.00"),
            delivery_fee: Money::from("5.00"),
            tip: Money::from("3.00"),
            federal_fee: Money::from("1.00"),
            local_ops_fee: Money::from("2.50"),
            processing_fee: Money::from("1.36"),
        };
        assert_eq!(b.total(), Money::from("37.86"));
    }

    #[test]
    fn display_shows_six_lines() {
        let b = PricingBreakdown {
            food_total: Money::from("10.00"),
            delivery_fee: Money::from("3.00"),
            tip: Money::from("0.00"),
            federal_fee: Money::from("1.00"),
            local_ops_fee: Money::from("2.00"),
            processing_fee: Money::from("0.76"),
        };
        let output = b.to_string();
        // 6 receipt lines + 1 total line
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 7);
        assert!(lines[0].starts_with("Food Total:"));
        assert!(lines[5].starts_with("Processing Fee:"));
        assert!(lines[6].starts_with("Total:"));
    }

    #[test]
    fn serde_roundtrip() {
        let b = PricingBreakdown {
            food_total: Money::from("25.00"),
            delivery_fee: Money::from("5.00"),
            tip: Money::from("3.00"),
            federal_fee: Money::from("1.00"),
            local_ops_fee: Money::from("2.50"),
            processing_fee: Money::from("1.36"),
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: PricingBreakdown = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total(), b.total());
    }

    // --- calculate_pricing tests ---

    #[test]
    fn calc_basic_order() {
        // $25 food, $5 delivery, $3 tip, $2.50 local ops
        let b = calculate_pricing(
            Money::from("25.00"),
            Money::from("5.00"),
            Money::from("3.00"),
            Money::from("2.50"),
        );
        assert_eq!(b.food_total, Money::from("25.00"));
        assert_eq!(b.delivery_fee, Money::from("5.00"));
        assert_eq!(b.tip, Money::from("3.00"));
        assert_eq!(b.federal_fee, Money::from("1.00"));
        assert_eq!(b.local_ops_fee, Money::from("2.50"));
        // processing = (25+5+3+1+2.50) * 0.029 + 0.30 = 36.50 * 0.029 + 0.30 = 1.0585 + 0.30 = 1.3585 → $1.36
        assert_eq!(b.processing_fee, Money::from("1.36"));
    }

    #[test]
    fn calc_zero_tip() {
        let b = calculate_pricing(
            Money::from("20.00"),
            Money::from("4.00"),
            Money::from("0.00"),
            Money::from("2.00"),
        );
        assert_eq!(b.tip, Money::from("0.00"));
        // processing = (20+4+0+1+2) * 0.029 + 0.30 = 27 * 0.029 + 0.30 = 0.783 + 0.30 = 1.083 → $1.08
        assert_eq!(b.processing_fee, Money::from("1.08"));
    }

    #[test]
    fn calc_zero_delivery() {
        let b = calculate_pricing(
            Money::from("15.00"),
            Money::from("0.00"),
            Money::from("2.00"),
            Money::from("1.50"),
        );
        assert_eq!(b.delivery_fee, Money::from("0.00"));
        // processing = (15+0+2+1+1.50) * 0.029 + 0.30 = 19.50 * 0.029 + 0.30 = 0.5655 + 0.30 = 0.8655 → $0.87
        assert_eq!(b.processing_fee, Money::from("0.87"));
    }

    #[test]
    fn calc_large_order() {
        let b = calculate_pricing(
            Money::from("250.00"),
            Money::from("10.00"),
            Money::from("50.00"),
            Money::from("3.00"),
        );
        assert_eq!(b.federal_fee, Money::from("1.00"));
        // processing = (250+10+50+1+3) * 0.029 + 0.30 = 314 * 0.029 + 0.30 = 9.106 + 0.30 = 9.406 → $9.41
        assert_eq!(b.processing_fee, Money::from("9.41"));
    }

    #[test]
    fn calc_federal_fee_always_one_dollar() {
        let b1 = calculate_pricing(
            Money::from("5.00"),
            Money::from("0.00"),
            Money::from("0.00"),
            Money::from("1.00"),
        );
        let b2 = calculate_pricing(
            Money::from("500.00"),
            Money::from("20.00"),
            Money::from("100.00"),
            Money::from("5.00"),
        );
        assert_eq!(b1.federal_fee, Money::from("1.00"));
        assert_eq!(b2.federal_fee, Money::from("1.00"));
    }

    #[test]
    fn calc_rounding_to_cents() {
        // Pick values that produce a processing fee needing rounding
        let b = calculate_pricing(
            Money::from("11.11"),
            Money::from("3.33"),
            Money::from("2.22"),
            Money::from("1.50"),
        );
        // processing = (11.11+3.33+2.22+1+1.50) * 0.029 + 0.30 = 19.16 * 0.029 + 0.30 = 0.55564 + 0.30 = 0.85564 → $0.86
        assert_eq!(b.processing_fee, Money::from("0.86"));
    }
}
