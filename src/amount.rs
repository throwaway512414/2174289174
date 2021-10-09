use std::{
    convert::TryFrom,
    fmt::Display,
    ops::{AddAssign, SubAssign},
};

use rust_decimal::prelude::*;
use serde::{de, Deserialize, Deserializer, Serialize};

/// A wrapper type for `rust_decimal::Decimal` to add additional constraints:
/// - The scale is no more than 4
/// - The value is nonnegative when created
///
/// As money amounts are specified in decimal it is necesarry to use a type that
/// can handle that. For example using `f32` cannot accurately represent decimals
/// and would therefore lead to rounding errors.
///
/// # Note
///
/// This type only guarantees that the [`Amount`] is nonnegative when deserialized.
/// This ensures that someone can not try to withdraw a negative amount for example.
/// It is not clear from the requirements wether the fields of [`Account`] (`total`, `fund`, etc)
/// can be negative or should always be nonnegative.
///
/// # Panics
///
/// The user of [`Amount`] must ensure that any mutable operation does not
/// make the resulting value overflow, because then it will panic.
#[derive(Debug, Serialize, Clone, Copy)]
pub struct Amount(Decimal);

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Amount {
    pub fn new(num: i64, scale: u32) -> Result<Self, String> {
        Self::try_from(Decimal::new(num, scale))
    }

    pub fn zero() -> Self {
        Self(Decimal::zero())
    }
}

impl TryFrom<Decimal> for Amount {
    type Error = String;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        if value.is_sign_negative() {
            return Err(format!(
                "`{}` is not a valid amount. It needs to be a nonnegative decimal number.",
                value
            ));
        }

        if value.scale() > 4 {
            return Err(format!(
                    "`{}` is not a valid amount. It needs to have a precision of no more than four places past the decimal.",
                    value
                ));
        }

        Ok(Amount(value))
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        // Assume no overflow
        self.0 += rhs.0;
    }
}

impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        // Assume no overflow
        self.0 -= rhs.0;
    }
}

impl PartialEq for Amount {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Amount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Amount, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: Decimal = Deserialize::deserialize(deserializer)?;

        Amount::try_from(val).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_valid_decimal_amounts() {
        let values = [
            // Zero
            Decimal::zero(),
            // Other examples
            Decimal::new(1, 1),
            Decimal::new(1, 2),
            Decimal::new(12, 3),
            Decimal::new(123, 4),
            Decimal::new(99999, 4),
        ];
        for value in values {
            assert!(Amount::try_from(value).is_ok());
        }
    }

    #[test]
    fn it_rejects_invalid_decimal_amounts() {
        let values = [
            // Negative
            Decimal::new(-1, 1),
            // Precision
            Decimal::new(1, 5),
            Decimal::new(-1, 5),
            Decimal::new(12, 6),
        ];
        for value in values {
            assert!(Amount::try_from(value).is_err());
        }
    }
}
