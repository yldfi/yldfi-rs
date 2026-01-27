//! Wei/Gwei/Ether conversion utilities
//!
//! Provides functions for converting between different Ethereum unit denominations.
//! All conversions use string representations to avoid floating-point precision issues.
//!
//! # Wei Newtype
//!
//! The `Wei` type provides a validated wrapper for token amounts in their smallest unit:
//!
//! ```
//! use yldfi_common::units::Wei;
//! use std::str::FromStr;
//!
//! // Parse from string
//! let amount = Wei::from_str("1000000000000000000").unwrap();
//! assert_eq!(amount.to_ether(), "1");
//!
//! // Create from u128
//! let amount = Wei::from_u128(1_000_000_000_000_000_000u128);
//! assert_eq!(amount.to_ether(), "1");
//!
//! // Arithmetic
//! let a = Wei::from_u128(100);
//! let b = Wei::from_u128(50);
//! assert!(a > b);
//! ```

use std::fmt;
use std::str::FromStr;

/// Common decimal places for Ethereum units
pub const WEI_DECIMALS: u8 = 0;
pub const GWEI_DECIMALS: u8 = 9;
pub const ETHER_DECIMALS: u8 = 18;

/// Error type for unit conversion failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitsError {
    /// Invalid number format
    InvalidNumber(String),
    /// Too many decimal places
    TooManyDecimals { max: u8, found: usize },
    /// Overflow during conversion
    Overflow,
}

impl fmt::Display for UnitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumber(s) => write!(f, "invalid number: {s}"),
            Self::TooManyDecimals { max, found } => {
                write!(f, "too many decimal places: max {max}, found {found}")
            }
            Self::Overflow => write!(f, "overflow during conversion"),
        }
    }
}

impl std::error::Error for UnitsError {}

// ============================================================================
// Wei Newtype
// ============================================================================

/// A validated token amount in the smallest unit (wei for ETH).
///
/// This type guarantees that the stored value is a valid non-negative integer.
/// It uses `u128` internally which can represent up to ~340 billion ETH
/// (or any token with 18 decimals), which is more than sufficient for all
/// practical use cases.
///
/// # Example
///
/// ```
/// use yldfi_common::units::Wei;
/// use std::str::FromStr;
///
/// let one_eth = Wei::from_str("1000000000000000000").unwrap();
/// assert_eq!(one_eth.to_ether(), "1");
/// assert_eq!(one_eth.as_u128(), 1_000_000_000_000_000_000u128);
///
/// // Safe construction from known values
/// let amount = Wei::from_u128(500_000_000); // 0.5 Gwei
/// assert_eq!(amount.to_gwei(), "0.5");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Wei(u128);

impl Wei {
    /// Create a new Wei amount from a u128.
    #[must_use]
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    /// Create a Wei amount from a u64.
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value as u128)
    }

    /// Create a Wei amount from a string, validating the format.
    ///
    /// Returns `None` if the string is not a valid non-negative integer.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        value.trim().parse::<u128>().ok().map(Self)
    }

    /// Parse a decimal string (like "1.5" ETH) into Wei.
    ///
    /// # Arguments
    /// * `value` - The decimal value (e.g., "1.5")
    /// * `decimals` - The number of decimal places (18 for ETH, 6 for USDC)
    pub fn from_decimal(value: &str, decimals: u8) -> Result<Self, UnitsError> {
        let wei_str = parse_units(value, decimals)?;
        // Handle negative result from parse_units
        if wei_str.starts_with('-') {
            return Err(UnitsError::InvalidNumber(
                "negative values not allowed".to_string(),
            ));
        }
        wei_str
            .parse::<u128>()
            .map(Self)
            .map_err(|_| UnitsError::Overflow)
    }

    /// Create Wei from an ether amount string.
    ///
    /// # Example
    ///
    /// ```
    /// use yldfi_common::units::Wei;
    ///
    /// let amount = Wei::from_ether("1.5").unwrap();
    /// assert_eq!(amount.as_u128(), 1_500_000_000_000_000_000u128);
    /// ```
    pub fn from_ether(ether: &str) -> Result<Self, UnitsError> {
        Self::from_decimal(ether, ETHER_DECIMALS)
    }

    /// Create Wei from a gwei amount string.
    ///
    /// # Example
    ///
    /// ```
    /// use yldfi_common::units::Wei;
    ///
    /// let amount = Wei::from_gwei("30.5").unwrap();
    /// assert_eq!(amount.as_u128(), 30_500_000_000u128);
    /// ```
    pub fn from_gwei(gwei: &str) -> Result<Self, UnitsError> {
        Self::from_decimal(gwei, GWEI_DECIMALS)
    }

    /// Get the raw u128 value.
    #[must_use]
    pub const fn as_u128(&self) -> u128 {
        self.0
    }

    /// Get the value as a string.
    #[must_use]
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }

    /// Convert to a decimal string with the given number of decimal places.
    #[must_use]
    pub fn to_decimal(&self, decimals: u8) -> String {
        format_units(&self.0.to_string(), decimals)
    }

    /// Convert to ether string representation.
    #[must_use]
    pub fn to_ether(&self) -> String {
        self.to_decimal(ETHER_DECIMALS)
    }

    /// Convert to gwei string representation.
    #[must_use]
    pub fn to_gwei(&self) -> String {
        self.to_decimal(GWEI_DECIMALS)
    }

    /// Format as human-readable string with unit suffix.
    #[must_use]
    pub fn to_human(&self) -> String {
        format_wei_human(&self.0.to_string())
    }

    /// Check if the amount is zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// The zero amount.
    pub const ZERO: Self = Self(0);

    /// Checked addition.
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    /// Checked subtraction.
    #[must_use]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    /// Checked multiplication.
    #[must_use]
    pub fn checked_mul(self, rhs: u128) -> Option<Self> {
        self.0.checked_mul(rhs).map(Self)
    }

    /// Checked division.
    #[must_use]
    pub fn checked_div(self, rhs: u128) -> Option<Self> {
        self.0.checked_div(rhs).map(Self)
    }

    /// Saturating addition (caps at `u128::MAX` instead of overflowing).
    #[must_use]
    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Saturating subtraction (floors at 0 instead of underflowing).
    #[must_use]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl fmt::Display for Wei {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Wei {
    type Err = WeiParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or(WeiParseError)
    }
}

impl From<u128> for Wei {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<u64> for Wei {
    fn from(value: u64) -> Self {
        Self(u128::from(value))
    }
}

impl From<Wei> for u128 {
    fn from(wei: Wei) -> Self {
        wei.0
    }
}

impl AsRef<u128> for Wei {
    fn as_ref(&self) -> &u128 {
        &self.0
    }
}

/// Error returned when parsing an invalid Wei amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeiParseError;

impl fmt::Display for WeiParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Wei amount")
    }
}

impl std::error::Error for WeiParseError {}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Convert a decimal string to wei (smallest unit).
///
/// # Arguments
/// * `value` - The value to convert (e.g., "1.5")
/// * `decimals` - The number of decimal places (18 for ETH, 6 for USDC, etc.)
///
/// # Examples
///
/// ```
/// use yldfi_common::units::to_wei;
///
/// // 1 ETH = 10^18 wei
/// assert_eq!(to_wei("1", 18).unwrap(), "1000000000000000000");
///
/// // 1.5 ETH
/// assert_eq!(to_wei("1.5", 18).unwrap(), "1500000000000000000");
///
/// // 0.001 ETH
/// assert_eq!(to_wei("0.001", 18).unwrap(), "1000000000000000");
///
/// // 100 USDC (6 decimals)
/// assert_eq!(to_wei("100", 6).unwrap(), "100000000");
/// ```
pub fn to_wei(value: &str, decimals: u8) -> Result<String, UnitsError> {
    parse_units(value, decimals)
}

/// Convert wei to a decimal string.
///
/// # Arguments
/// * `wei` - The wei amount as a string
/// * `decimals` - The number of decimal places (18 for ETH, 6 for USDC, etc.)
///
/// # Examples
///
/// ```
/// use yldfi_common::units::from_wei;
///
/// // 10^18 wei = 1 ETH
/// assert_eq!(from_wei("1000000000000000000", 18), "1");
///
/// // 1.5 ETH
/// assert_eq!(from_wei("1500000000000000000", 18), "1.5");
///
/// // Small amount
/// assert_eq!(from_wei("1000000000000000", 18), "0.001");
///
/// // 100 USDC
/// assert_eq!(from_wei("100000000", 6), "100");
/// ```
#[must_use] 
pub fn from_wei(wei: &str, decimals: u8) -> String {
    format_units(wei, decimals)
}

/// Convert ether to wei.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::ether_to_wei;
///
/// assert_eq!(ether_to_wei("1").unwrap(), "1000000000000000000");
/// assert_eq!(ether_to_wei("0.5").unwrap(), "500000000000000000");
/// ```
pub fn ether_to_wei(ether: &str) -> Result<String, UnitsError> {
    to_wei(ether, ETHER_DECIMALS)
}

/// Convert wei to ether.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::wei_to_ether;
///
/// assert_eq!(wei_to_ether("1000000000000000000"), "1");
/// assert_eq!(wei_to_ether("500000000000000000"), "0.5");
/// ```
#[must_use] 
pub fn wei_to_ether(wei: &str) -> String {
    from_wei(wei, ETHER_DECIMALS)
}

/// Convert gwei to wei.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::gwei_to_wei;
///
/// assert_eq!(gwei_to_wei("1").unwrap(), "1000000000");
/// assert_eq!(gwei_to_wei("30.5").unwrap(), "30500000000");
/// ```
pub fn gwei_to_wei(gwei: &str) -> Result<String, UnitsError> {
    to_wei(gwei, GWEI_DECIMALS)
}

/// Convert wei to gwei.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::wei_to_gwei;
///
/// assert_eq!(wei_to_gwei("1000000000"), "1");
/// assert_eq!(wei_to_gwei("30500000000"), "30.5");
/// ```
#[must_use] 
pub fn wei_to_gwei(wei: &str) -> String {
    from_wei(wei, GWEI_DECIMALS)
}

/// Convert ether to gwei.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::ether_to_gwei;
///
/// assert_eq!(ether_to_gwei("1").unwrap(), "1000000000");
/// assert_eq!(ether_to_gwei("0.001").unwrap(), "1000000");
/// ```
pub fn ether_to_gwei(ether: &str) -> Result<String, UnitsError> {
    // First convert ether to wei, then wei to gwei
    let wei = ether_to_wei(ether)?;
    // gwei = wei / 10^9, which is just removing 9 zeros
    // But we need to handle this properly
    let gwei_decimals = ETHER_DECIMALS - GWEI_DECIMALS;
    Ok(format_units(&wei, gwei_decimals))
}

/// Convert gwei to ether.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::gwei_to_ether;
///
/// assert_eq!(gwei_to_ether("1000000000"), "1");
/// assert_eq!(gwei_to_ether("1000000"), "0.001");
/// ```
#[must_use] 
pub fn gwei_to_ether(gwei: &str) -> String {
    // gwei has 9 decimals relative to ether
    format_units(gwei, GWEI_DECIMALS)
}

/// Parse a decimal string into its smallest unit representation.
///
/// This is a more general version of `to_wei` that works with any decimal precision.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::parse_units;
///
/// // Parse 1.5 with 18 decimals
/// assert_eq!(parse_units("1.5", 18).unwrap(), "1500000000000000000");
///
/// // Parse integer
/// assert_eq!(parse_units("100", 6).unwrap(), "100000000");
///
/// // Parse with leading zeros
/// assert_eq!(parse_units("0.000001", 6).unwrap(), "1");
/// ```
pub fn parse_units(value: &str, decimals: u8) -> Result<String, UnitsError> {
    let value = value.trim();

    // Handle negative numbers
    let (is_negative, value) = if let Some(v) = value.strip_prefix('-') {
        (true, v)
    } else {
        (false, value)
    };

    // Split into integer and fractional parts
    let (integer_part, fractional_part) = match value.split_once('.') {
        Some((int, frac)) => (int, frac),
        None => (value, ""),
    };

    // Validate integer part
    if !integer_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(UnitsError::InvalidNumber(value.to_string()));
    }

    // Validate and process fractional part
    if !fractional_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(UnitsError::InvalidNumber(value.to_string()));
    }

    let frac_len = fractional_part.len();
    if frac_len > decimals as usize {
        return Err(UnitsError::TooManyDecimals {
            max: decimals,
            found: frac_len,
        });
    }

    // Build the result: integer_part + fractional_part + padding zeros
    let mut result = String::new();

    // Handle leading zeros in integer part
    let integer_part = integer_part.trim_start_matches('0');
    if integer_part.is_empty() && fractional_part.is_empty() {
        return Ok("0".to_string());
    }

    result.push_str(integer_part);
    result.push_str(fractional_part);

    // Pad with zeros to reach the correct decimal places
    let zeros_to_add = decimals as usize - frac_len;
    result.push_str(&"0".repeat(zeros_to_add));

    // Remove leading zeros from result
    let result = result.trim_start_matches('0');
    let result = if result.is_empty() { "0" } else { result };

    if is_negative && result != "0" {
        Ok(format!("-{result}"))
    } else {
        Ok(result.to_string())
    }
}

/// Format a smallest-unit value as a decimal string.
///
/// This is a more general version of `from_wei` that works with any decimal precision.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::format_units;
///
/// // Format with 18 decimals
/// assert_eq!(format_units("1500000000000000000", 18), "1.5");
///
/// // Format integer result
/// assert_eq!(format_units("100000000", 6), "100");
///
/// // Format small value
/// assert_eq!(format_units("1", 6), "0.000001");
///
/// // Zero
/// assert_eq!(format_units("0", 18), "0");
/// ```
#[must_use] 
pub fn format_units(value: &str, decimals: u8) -> String {
    let value = value.trim();

    // Handle negative numbers
    let (is_negative, value) = if let Some(v) = value.strip_prefix('-') {
        (true, v)
    } else {
        (false, value)
    };

    // Handle zero
    if value.chars().all(|c| c == '0') {
        return "0".to_string();
    }

    let decimals = decimals as usize;

    // Pad with leading zeros if necessary
    let padded = if value.len() <= decimals {
        format!("{:0>width$}", value, width = decimals + 1)
    } else {
        value.to_string()
    };

    // Split into integer and fractional parts
    let split_point = padded.len() - decimals;
    let integer_part = &padded[..split_point];
    let fractional_part = &padded[split_point..];

    // Remove leading zeros from integer part
    let integer_part = integer_part.trim_start_matches('0');
    let integer_part = if integer_part.is_empty() {
        "0"
    } else {
        integer_part
    };

    // Remove trailing zeros from fractional part
    let fractional_part = fractional_part.trim_end_matches('0');

    let result = if fractional_part.is_empty() {
        integer_part.to_string()
    } else {
        format!("{integer_part}.{fractional_part}")
    };

    if is_negative {
        format!("-{result}")
    } else {
        result
    }
}

/// Format wei as a human-readable string with unit suffix.
///
/// Automatically chooses the best unit (Wei, Gwei, or ETH) for readability.
///
/// # Examples
///
/// ```
/// use yldfi_common::units::format_wei_human;
///
/// assert_eq!(format_wei_human("1000000000000000000"), "1 ETH");
/// assert_eq!(format_wei_human("1500000000000000000"), "1.5 ETH");
/// assert_eq!(format_wei_human("30000000000"), "30 Gwei");
/// assert_eq!(format_wei_human("1000"), "1000 Wei");
/// ```
#[must_use] 
pub fn format_wei_human(wei: &str) -> String {
    let wei = wei.trim();

    // Try to parse as u128 for comparison
    let wei_val: u128 = match wei.parse() {
        Ok(v) => v,
        Err(_) => return format!("{wei} Wei"), // Fallback for very large numbers
    };

    const GWEI: u128 = 1_000_000_000;
    const ETHER: u128 = 1_000_000_000_000_000_000;

    if wei_val >= ETHER {
        let formatted = wei_to_ether(wei);
        format!("{formatted} ETH")
    } else if wei_val >= GWEI {
        let formatted = wei_to_gwei(wei);
        format!("{formatted} Gwei")
    } else {
        format!("{wei} Wei")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_wei_integers() {
        assert_eq!(to_wei("1", 18).unwrap(), "1000000000000000000");
        assert_eq!(to_wei("100", 18).unwrap(), "100000000000000000000");
        assert_eq!(to_wei("0", 18).unwrap(), "0");
    }

    #[test]
    fn test_to_wei_decimals() {
        assert_eq!(to_wei("1.5", 18).unwrap(), "1500000000000000000");
        assert_eq!(to_wei("0.5", 18).unwrap(), "500000000000000000");
        assert_eq!(to_wei("0.001", 18).unwrap(), "1000000000000000");
        assert_eq!(to_wei("0.000000000000000001", 18).unwrap(), "1");
    }

    #[test]
    fn test_to_wei_usdc() {
        // USDC has 6 decimals
        assert_eq!(to_wei("1", 6).unwrap(), "1000000");
        assert_eq!(to_wei("100.5", 6).unwrap(), "100500000");
        assert_eq!(to_wei("0.000001", 6).unwrap(), "1");
    }

    #[test]
    fn test_to_wei_too_many_decimals() {
        let result = to_wei("1.0000001", 6);
        assert!(matches!(result, Err(UnitsError::TooManyDecimals { .. })));
    }

    #[test]
    fn test_from_wei() {
        assert_eq!(from_wei("1000000000000000000", 18), "1");
        assert_eq!(from_wei("1500000000000000000", 18), "1.5");
        assert_eq!(from_wei("500000000000000000", 18), "0.5");
        assert_eq!(from_wei("1", 18), "0.000000000000000001");
        assert_eq!(from_wei("0", 18), "0");
    }

    #[test]
    fn test_ether_gwei_conversions() {
        assert_eq!(ether_to_wei("1").unwrap(), "1000000000000000000");
        assert_eq!(wei_to_ether("1000000000000000000"), "1");

        assert_eq!(gwei_to_wei("1").unwrap(), "1000000000");
        assert_eq!(wei_to_gwei("1000000000"), "1");

        assert_eq!(ether_to_gwei("1").unwrap(), "1000000000");
        assert_eq!(gwei_to_ether("1000000000"), "1");
    }

    #[test]
    fn test_format_wei_human() {
        assert_eq!(format_wei_human("1000000000000000000"), "1 ETH");
        assert_eq!(format_wei_human("1500000000000000000"), "1.5 ETH");
        assert_eq!(format_wei_human("30000000000"), "30 Gwei");
        assert_eq!(format_wei_human("1000"), "1000 Wei");
        assert_eq!(format_wei_human("0"), "0 Wei");
    }

    #[test]
    fn test_negative_values() {
        assert_eq!(to_wei("-1.5", 18).unwrap(), "-1500000000000000000");
        assert_eq!(from_wei("-1500000000000000000", 18), "-1.5");
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(to_wei("01.5", 18).unwrap(), "1500000000000000000");
        assert_eq!(to_wei("001", 18).unwrap(), "1000000000000000000");
    }

    #[test]
    fn test_invalid_input() {
        assert!(to_wei("abc", 18).is_err());
        assert!(to_wei("1.2.3", 18).is_err());
        assert!(to_wei("1e18", 18).is_err());
    }

    // ========================================================================
    // Wei Newtype Tests
    // ========================================================================

    #[test]
    fn test_wei_from_str() {
        let wei: Wei = "1000000000000000000".parse().unwrap();
        assert_eq!(wei.as_u128(), 1_000_000_000_000_000_000u128);

        // Invalid string
        assert!("abc".parse::<Wei>().is_err());
        assert!("-100".parse::<Wei>().is_err());
    }

    #[test]
    fn test_wei_from_ether() {
        let wei = Wei::from_ether("1").unwrap();
        assert_eq!(wei.as_u128(), 1_000_000_000_000_000_000u128);

        let wei = Wei::from_ether("1.5").unwrap();
        assert_eq!(wei.as_u128(), 1_500_000_000_000_000_000u128);

        let wei = Wei::from_ether("0.001").unwrap();
        assert_eq!(wei.as_u128(), 1_000_000_000_000_000u128);
    }

    #[test]
    fn test_wei_from_gwei() {
        let wei = Wei::from_gwei("1").unwrap();
        assert_eq!(wei.as_u128(), 1_000_000_000u128);

        let wei = Wei::from_gwei("30.5").unwrap();
        assert_eq!(wei.as_u128(), 30_500_000_000u128);
    }

    #[test]
    fn test_wei_conversions() {
        let wei = Wei::from_u128(1_500_000_000_000_000_000);

        assert_eq!(wei.to_ether(), "1.5");
        assert_eq!(wei.to_gwei(), "1500000000");
        assert_eq!(wei.to_human(), "1.5 ETH");
    }

    #[test]
    fn test_wei_arithmetic() {
        let a = Wei::from_u128(100);
        let b = Wei::from_u128(50);

        // Checked operations
        assert_eq!(a.checked_add(b), Some(Wei::from_u128(150)));
        assert_eq!(a.checked_sub(b), Some(Wei::from_u128(50)));
        assert_eq!(b.checked_sub(a), None); // Would underflow
        assert_eq!(a.checked_mul(2), Some(Wei::from_u128(200)));
        assert_eq!(a.checked_div(2), Some(Wei::from_u128(50)));
        assert_eq!(a.checked_div(0), None);

        // Saturating operations
        assert_eq!(b.saturating_sub(a), Wei::ZERO);
    }

    #[test]
    fn test_wei_comparisons() {
        let a = Wei::from_u128(100);
        let b = Wei::from_u128(50);
        let c = Wei::from_u128(100);

        assert!(a > b);
        assert!(b < a);
        assert!(a >= c);
        assert!(a <= c);
        assert_eq!(a, c);
        assert_ne!(a, b);
    }

    #[test]
    fn test_wei_zero() {
        let zero = Wei::ZERO;
        assert!(zero.is_zero());
        assert_eq!(zero.as_u128(), 0);

        let non_zero = Wei::from_u128(1);
        assert!(!non_zero.is_zero());
    }

    #[test]
    fn test_wei_display() {
        let wei = Wei::from_u128(12345);
        assert_eq!(format!("{}", wei), "12345");
    }

    #[test]
    fn test_wei_negative_rejected() {
        // from_decimal should reject negative values
        let result = Wei::from_decimal("-1", 18);
        assert!(result.is_err());
    }

    #[test]
    fn test_wei_from_decimal_usdc() {
        // USDC has 6 decimals
        let amount = Wei::from_decimal("100.5", 6).unwrap();
        assert_eq!(amount.as_u128(), 100_500_000);
        assert_eq!(amount.to_decimal(6), "100.5");
    }
}
