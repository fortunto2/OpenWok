use crate::money::Money;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The 6-line open-book receipt — zero hidden markup.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
