#![allow(dead_code)]

use core::fmt;

use crate::color::model::Color;

/// An error caused by parsing an invalid color string slice.
///
/// # Variants
///
/// - `Empty` - The given string slice was empty or all whitespace.
/// - `InvalidLength` - The given string slice had an invalid length.
/// - `InvalidHex` - The given string slice was not a valid hex representation.
///
/// # Examples
///
/// ```
/// use codimate::color::{ColorParseError, parse_color};
///
/// let hex = "#fff";
/// let color = parse_color(hex)
/// match color {
///     Ok(v) => println!("Good color value: {}", color.into_hex6()),
///     Err(e) => println!("Error parsing color: {}", e),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    Empty,
    InvalidLength,
    InvalidHex,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ColorParseError::*;
        let msg = match self {
            Empty => "empty color string",
            InvalidLength => "invalid hex length",
            InvalidHex => "invalid hex digits",
        };
        f.write_str(msg)
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ColorParseError {}

/// Parse a hex color from a string.
///
/// The allowed formats are:
/// * #RGB
/// * #RGBA
/// * #RRGGBB
/// * #RRGGBBAA
///
/// # Arguments
///
/// - `hex` (`&str`) - The hex value to parse.
///
/// # Returns
///
/// - `Result<Color, ColorParseError>` - The result of parsing.
fn parse_hex(hex: &str) -> Result<Color, ColorParseError> {
    use ColorParseError::*;

    let nibble = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    };

    let bytes = hex.as_bytes();
    let (r, g, b, a) = match bytes.len() {
        3 => {
            // #RGB
            let r = nibble(bytes[0]).ok_or(InvalidHex)?;
            let g = nibble(bytes[1]).ok_or(InvalidHex)?;
            let b = nibble(bytes[2]).ok_or(InvalidHex)?;

            (r * 17, g * 17, b * 17, 255)
        }
        4 => {
            // #RGBA
            let r = nibble(bytes[0]).ok_or(InvalidHex)?;
            let g = nibble(bytes[1]).ok_or(InvalidHex)?;
            let b = nibble(bytes[2]).ok_or(InvalidHex)?;
            let a = nibble(bytes[3]).ok_or(InvalidHex)?;

            (r * 17, g * 17, b * 17, a * 17)
        }
        6 => {
            // #RRGGBB
            let nibble2 = |hi: u8, lo: u8| -> Result<u8, ColorParseError> {
                let h = nibble(hi).ok_or(InvalidHex)?;
                let l = nibble(lo).ok_or(InvalidHex)?;

                Ok(h << 4 | l)
            };

            (
                nibble2(bytes[0], bytes[1])?,
                nibble2(bytes[2], bytes[3])?,
                nibble2(bytes[4], bytes[5])?,
                255,
            )
        }
        8 => {
            // #RRGGBBAA
            let nibble2 = |hi: u8, lo: u8| -> Result<u8, ColorParseError> {
                let h = nibble(hi).ok_or(InvalidHex)?;
                let l = nibble(lo).ok_or(InvalidHex)?;

                Ok(h << 4 | l)
            };

            (
                nibble2(bytes[0], bytes[1])?,
                nibble2(bytes[2], bytes[3])?,
                nibble2(bytes[4], bytes[5])?,
                nibble2(bytes[6], bytes[7])?,
            )
        }
        _ => return Err(InvalidLength),
    };

    Ok(Color::from_rgba([r, g, b, a]))
}

/// Parse a color from a string slice.
/// This function supports:
/// * Hex string slices
///
/// # Arguments
///
/// - `mut s` (`&str`) - The string slice to parse.
///
/// # Returns
///
/// - `Result<Color, ColorParseError>` - The result of the parse.
///
/// # Examples
///
/// ```
/// use codimate::color::parse_color;
///
/// let hex = "#fff";
/// let color = parse_color(hex)
/// match color {
///     Ok(v) => println!("Good color value: {}", color.into_hex6()),
///     Err(e) => println!("Error parsing color: {}", e),
/// }
/// ```
pub fn parse_color(mut s: &str) -> Result<Color, ColorParseError> {
    use ColorParseError::*;

    if s.trim().is_empty() {
        return Err(Empty);
    }
    s = s.trim();

    // Hex-like
    if let Some(rest) = s.strip_prefix('#') {
        let hex = rest.trim();
        return parse_hex(hex);
    }

    Err(InvalidHex)
}

impl core::str::FromStr for Color {
    type Err = ColorParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_color(s)
    }
}
impl TryFrom<&str> for Color {
    type Error = ColorParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_color(value)
    }
}
