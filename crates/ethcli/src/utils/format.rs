//! Shared formatting utilities for numbers, tokens, ETH values, and tables.

/// Add thousands separators to a numeric string.
///
/// # Examples
/// ```
/// use ethcli::utils::format::with_thousands_sep;
/// assert_eq!(with_thousands_sep("1234567"), "1,234,567");
/// assert_eq!(with_thousands_sep("123"), "123");
/// ```
pub fn with_thousands_sep(s: &str) -> String {
    // Handle negative numbers
    let (sign, num) = if let Some(stripped) = s.strip_prefix('-') {
        ("-", stripped)
    } else {
        ("", s)
    };

    let bytes = num.as_bytes();
    let len = bytes.len();

    if len <= 3 {
        return s.to_string();
    }

    let mut result = String::with_capacity(len + (len - 1) / 3);
    let first_group = len % 3;

    if first_group > 0 {
        result.push_str(&num[..first_group]);
        if len > first_group {
            result.push(',');
        }
    }

    for (i, chunk) in num.as_bytes()[first_group..].chunks(3).enumerate() {
        if i > 0 {
            result.push(',');
        }
        // SAFETY: chunk is from s.as_bytes() which is guaranteed valid UTF-8
        result.push_str(std::str::from_utf8(chunk).expect("input was valid UTF-8"));
    }

    format!("{}{}", sign, result)
}

/// Format a u64 with thousands separators.
pub fn format_thousands(n: u64) -> String {
    with_thousands_sep(&n.to_string())
}

/// Format a raw token amount string with decimals and thousands separators.
///
/// # Arguments
/// * `raw` - The raw token amount as a string (e.g., "1000000000000000000")
/// * `decimals` - Number of decimal places (e.g., 18 for ETH, 6 for USDC)
///
/// # Examples
/// ```
/// use ethcli::utils::format::format_token_amount;
/// assert_eq!(format_token_amount("1000000000000000000", 18), "1");
/// assert_eq!(format_token_amount("1500000", 6), "1.5");
/// ```
pub fn format_token_amount(raw: &str, decimals: u8) -> String {
    let dec = decimals as usize;
    let len = raw.len();

    if len <= dec {
        let padded = format!("{:0>width$}", raw, width = dec);
        let trimmed = padded.trim_end_matches('0');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            format!("0.{}", trimmed)
        }
    } else {
        let integer_part = &raw[..len - dec];
        let decimal_part = &raw[len - dec..];
        let decimal_trimmed = decimal_part.trim_end_matches('0');

        let formatted_int = with_thousands_sep(integer_part);

        if decimal_trimmed.is_empty() {
            formatted_int
        } else {
            format!("{}.{}", formatted_int, decimal_trimmed)
        }
    }
}

/// Format wei amount to ETH with up to 4 decimal places.
///
/// # Examples
/// ```
/// use ethcli::utils::format::format_wei_to_eth;
/// assert_eq!(format_wei_to_eth("1000000000000000000"), "1.0");
/// assert_eq!(format_wei_to_eth("1500000000000000000"), "1.5");
/// ```
pub fn format_wei_to_eth(wei: &str) -> String {
    const ETH_DECIMALS: usize = 18;
    const DISPLAY_DECIMALS: usize = 4;

    let wei_len = wei.len();
    if wei_len <= ETH_DECIMALS {
        let padded = format!("{:0>width$}", wei, width = ETH_DECIMALS);
        let decimal = padded.trim_end_matches('0');
        if decimal.is_empty() {
            "0.0".to_string()
        } else {
            format!("0.{}", decimal)
        }
    } else {
        let integer_part = &wei[..wei_len - ETH_DECIMALS];
        let decimal_part = &wei[wei_len - ETH_DECIMALS..];
        let decimal_trimmed =
            decimal_part[..DISPLAY_DECIMALS.min(decimal_part.len())].trim_end_matches('0');
        if decimal_trimmed.is_empty() {
            format!("{}.0", integer_part)
        } else {
            format!("{}.{}", integer_part, decimal_trimmed)
        }
    }
}

/// Format a U256 value with decimal places.
pub fn format_u256_with_decimals(value: &alloy::primitives::U256, decimals: u8) -> String {
    format_token_amount(&value.to_string(), decimals)
}

/// Truncate a string to a maximum length, adding "..." if truncated.
///
/// This function is UTF-8 safe and will not panic on multi-byte characters.
/// The returned string will have at most `max_len` characters (not bytes).
pub fn truncate_str(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s.chars().take(max_len).collect()
    } else {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

/// Column alignment for table formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

/// A column definition for table formatting.
#[derive(Debug, Clone)]
pub struct Column {
    pub width: usize,
    pub align: Align,
}

impl Column {
    /// Create a left-aligned column with the given width.
    pub fn left(width: usize) -> Self {
        Self {
            width,
            align: Align::Left,
        }
    }

    /// Create a right-aligned column with the given width.
    pub fn right(width: usize) -> Self {
        Self {
            width,
            align: Align::Right,
        }
    }

    /// Format a value according to this column's alignment and width.
    pub fn format(&self, value: &str) -> String {
        let truncated = if value.len() > self.width {
            truncate_str(value, self.width)
        } else {
            value.to_string()
        };

        match self.align {
            Align::Left => format!("{:<width$}", truncated, width = self.width),
            Align::Right => format!("{:>width$}", truncated, width = self.width),
        }
    }
}

/// A simple table formatter for CLI output.
///
/// # Example
/// ```
/// use ethcli::utils::format::{Table, Column};
///
/// let table = Table::new(vec![
///     Column::left(12),
///     Column::right(14),
///     Column::right(10),
/// ]);
///
/// table.print_separator();
/// table.print_row(&["Source", "Price USD", "Status"]);
/// table.print_separator();
/// table.print_row(&["CoinGecko", "$1234.56", "OK"]);
/// table.print_separator();
/// ```
pub struct Table {
    columns: Vec<Column>,
    total_width: usize,
}

impl Table {
    /// Create a new table with the given column definitions.
    pub fn new(columns: Vec<Column>) -> Self {
        let total_width = columns.iter().map(|c| c.width).sum::<usize>() + columns.len() - 1;
        Self {
            columns,
            total_width,
        }
    }

    /// Print a row of values, one per column.
    pub fn print_row(&self, values: &[&str]) {
        let formatted: Vec<String> = self
            .columns
            .iter()
            .zip(values.iter())
            .map(|(col, val)| col.format(val))
            .collect();
        println!("{}", formatted.join(" "));
    }

    /// Print a separator line spanning the table width.
    pub fn print_separator(&self) {
        println!("{}", "-".repeat(self.total_width));
    }

    /// Print a title followed by a separator.
    pub fn print_title(&self, title: &str) {
        println!("{}", title);
        self.print_separator();
    }

    /// Get the total width of the table (for custom separators).
    pub fn width(&self) -> usize {
        self.total_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_thousands_sep() {
        assert_eq!(with_thousands_sep("123"), "123");
        assert_eq!(with_thousands_sep("1234"), "1,234");
        assert_eq!(with_thousands_sep("1234567"), "1,234,567");
        assert_eq!(with_thousands_sep("-1234567"), "-1,234,567");
    }

    #[test]
    fn test_format_thousands() {
        assert_eq!(format_thousands(123), "123");
        assert_eq!(format_thousands(1234), "1,234");
        assert_eq!(format_thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn test_format_token_amount() {
        // 1 ETH (18 decimals)
        assert_eq!(format_token_amount("1000000000000000000", 18), "1");
        // 1.5 USDC (6 decimals)
        assert_eq!(format_token_amount("1500000", 6), "1.5");
        // 0.5 ETH
        assert_eq!(format_token_amount("500000000000000000", 18), "0.5");
        // Large amount
        assert_eq!(
            format_token_amount("1234567000000000000000000", 18),
            "1,234,567"
        );
    }

    #[test]
    fn test_format_wei_to_eth() {
        assert_eq!(format_wei_to_eth("1000000000000000000"), "1.0");
        assert_eq!(format_wei_to_eth("1500000000000000000"), "1.5");
        assert_eq!(format_wei_to_eth("500000000000000000"), "0.5");
        assert_eq!(format_wei_to_eth("0"), "0.0");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
        assert_eq!(truncate_str("hi", 2), "hi");
        assert_eq!(truncate_str("hello", 3), "hel");
    }

    #[test]
    fn test_column_format() {
        let left = Column::left(10);
        assert_eq!(left.format("test"), "test      ");

        let right = Column::right(10);
        assert_eq!(right.format("test"), "      test");

        // Truncation
        let narrow = Column::left(5);
        assert_eq!(narrow.format("hello world"), "he...");
    }

    #[test]
    fn test_table_width() {
        let table = Table::new(vec![Column::left(10), Column::right(15), Column::right(8)]);
        // 10 + 15 + 8 + 2 spaces between = 35
        assert_eq!(table.width(), 35);
    }
}
