//! Shared table formatting utilities
//!
//! Provides consistent table output formatting across CLI commands.

use std::fmt::Display;

/// A simple table builder for CLI output
#[derive(Debug, Default)]
pub struct Table {
    /// Column headers
    headers: Vec<String>,
    /// Column widths (0 = auto)
    widths: Vec<usize>,
    /// Row data
    rows: Vec<Vec<String>>,
    /// Column alignments
    alignments: Vec<Alignment>,
}

/// Column alignment
#[derive(Debug, Clone, Copy, Default)]
pub enum Alignment {
    #[default]
    Left,
    Right,
    Center,
}

impl Table {
    /// Create a new table with the given headers
    pub fn new(headers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let headers: Vec<String> = headers.into_iter().map(|h| h.into()).collect();
        let len = headers.len();
        Self {
            headers,
            widths: vec![0; len],
            rows: Vec::new(),
            alignments: vec![Alignment::Left; len],
        }
    }

    /// Set column widths (0 = auto-calculate)
    pub fn with_widths(mut self, widths: impl IntoIterator<Item = usize>) -> Self {
        self.widths = widths.into_iter().collect();
        // Ensure widths vec matches headers length
        self.widths.resize(self.headers.len(), 0);
        self
    }

    /// Set column alignments
    pub fn with_alignments(mut self, alignments: impl IntoIterator<Item = Alignment>) -> Self {
        self.alignments = alignments.into_iter().collect();
        // Ensure alignments vec matches headers length
        self.alignments
            .resize(self.headers.len(), Alignment::Left);
        self
    }

    /// Add a row of data
    pub fn add_row(&mut self, row: impl IntoIterator<Item = impl Display>) {
        let row: Vec<String> = row.into_iter().map(|c| c.to_string()).collect();
        self.rows.push(row);
    }

    /// Add a row using a builder pattern
    pub fn row(mut self, row: impl IntoIterator<Item = impl Display>) -> Self {
        self.add_row(row);
        self
    }

    /// Calculate actual column widths
    fn calculate_widths(&self) -> Vec<usize> {
        let mut widths = self.widths.clone();

        // Start with header widths for auto columns
        for (i, header) in self.headers.iter().enumerate() {
            if i < widths.len() && widths[i] == 0 {
                widths[i] = header.len();
            }
        }

        // Update with max row widths for auto columns
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() && self.widths.get(i).copied().unwrap_or(0) == 0 {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        widths
    }

    /// Format a cell with alignment
    fn format_cell(value: &str, width: usize, alignment: Alignment) -> String {
        match alignment {
            Alignment::Left => format!("{:<width$}", value, width = width),
            Alignment::Right => format!("{:>width$}", value, width = width),
            Alignment::Center => format!("{:^width$}", value, width = width),
        }
    }

    /// Render the table to a string
    pub fn render(&self) -> String {
        let widths = self.calculate_widths();
        let mut output = String::new();

        // Header row
        let header_cells: Vec<String> = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let width = widths.get(i).copied().unwrap_or(h.len());
                let align = self.alignments.get(i).copied().unwrap_or(Alignment::Left);
                Self::format_cell(h, width, align)
            })
            .collect();
        output.push_str(&header_cells.join("  "));
        output.push('\n');

        // Separator
        let separator: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
        output.push_str(&separator.join("  "));
        output.push('\n');

        // Data rows
        for row in &self.rows {
            let cells: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let width = widths.get(i).copied().unwrap_or(cell.len());
                    let align = self.alignments.get(i).copied().unwrap_or(Alignment::Left);
                    Self::format_cell(cell, width, align)
                })
                .collect();
            output.push_str(&cells.join("  "));
            output.push('\n');
        }

        output
    }

    /// Print the table to stdout
    pub fn print(&self) {
        print!("{}", self.render());
    }

    /// Check if table has any data rows
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get the number of data rows
    pub fn len(&self) -> usize {
        self.rows.len()
    }
}

/// Format a number with thousands separators
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a float with specified decimal places
pub fn format_float(n: f64, decimals: usize) -> String {
    format!("{:.decimals$}", n, decimals = decimals)
}

/// Format a price in USD
pub fn format_usd(price: f64) -> String {
    if price >= 1.0 {
        format!("${:.2}", price)
    } else if price >= 0.0001 {
        format!("${:.6}", price)
    } else {
        format!("${:.10}", price)
    }
}

/// Format bytes as hex with 0x prefix, truncating if needed
pub fn format_hex_truncated(hex: &str, max_len: usize) -> String {
    if hex.len() <= max_len {
        hex.to_string()
    } else if max_len > 6 {
        format!("{}...{}", &hex[..max_len / 2], &hex[hex.len() - 3..])
    } else {
        format!("{}...", &hex[..max_len - 3])
    }
}

/// Format an address with optional truncation
pub fn format_address(address: &str, truncate: bool) -> String {
    if truncate && address.len() > 12 {
        format!("{}...{}", &address[..6], &address[address.len() - 4..])
    } else {
        address.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_basic() {
        let table = Table::new(["Name", "Value"])
            .row(["Alice", "100"])
            .row(["Bob", "200"]);

        let output = table.render();
        assert!(output.contains("Name"));
        assert!(output.contains("Value"));
        assert!(output.contains("Alice"));
        assert!(output.contains("100"));
        assert!(output.contains("Bob"));
        assert!(output.contains("200"));
    }

    #[test]
    fn test_table_alignment() {
        let table = Table::new(["Left", "Right", "Center"])
            .with_alignments([Alignment::Left, Alignment::Right, Alignment::Center])
            .row(["a", "b", "c"]);

        let output = table.render();
        assert!(output.contains("Left"));
        assert!(output.contains("Right"));
        assert!(output.contains("Center"));
    }

    #[test]
    fn test_table_empty() {
        let table = Table::new(["Col1", "Col2"]);
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(1000000000), "1,000,000,000");
    }

    #[test]
    fn test_format_usd() {
        assert_eq!(format_usd(1234.56), "$1234.56");
        assert_eq!(format_usd(0.123456), "$0.123456");
        assert_eq!(format_usd(0.0000001234), "$0.0000001234");
    }

    #[test]
    fn test_format_address() {
        let addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        assert_eq!(format_address(addr, false), addr);
        assert_eq!(format_address(addr, true), "0xA0b8...eB48");
    }

    #[test]
    fn test_format_hex_truncated() {
        let hex = "0x1234567890abcdef";
        assert_eq!(format_hex_truncated(hex, 20), hex);
        assert_eq!(format_hex_truncated(hex, 10), "0x123...def");
    }
}
