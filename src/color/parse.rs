#![allow(dead_code)]

use core::fmt;

use anyhow::Ok;

use crate::color::{ColorFloat, model::Color};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    Empty,
    InvalidLength,
    InvalidHex,
    InvalidFunc,
    OutOfRange,
    InvalidToken,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ColorParseError::*;
        let msg = match self {
            Empty => "empty color string",
            InvalidLength => "invalid hex length",
            InvalidHex => "invalid hex digits",
            InvalidFunc => "invalid function name",
            OutOfRange => "component out of range",
            InvalidToken => "invalid token found in function",
        };
        f.write_str(msg)
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ColorParseError {}

/// All types of tokens that can be found in a CSS color function signature.
///
/// Number can be represented as:
///     [+-]? [0-9]* ( "." [0-9]+ )? ["%"]?
/// ex: 10, 10.5, +10, -0.1, 50%, etc
#[derive(Clone, Debug, PartialEq, Eq)]
enum CssColorToken<'a> {
    Number(&'a str), // may have sign, decimal, %, etc
    Slash,
    Comma,
}

fn tokenize(input: &str) -> Result<Vec<CssColorToken<'_>>, ColorParseError> {
    use ColorParseError::InvalidToken;

    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            // skip whitespace
            b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
            }

            // start of a number: sign, digit, or '.'
            b'+' | b'-' | b'0'..=b'9' | b'.' => {
                let start = i;

                // optional sign
                if bytes[i] == b'+' || bytes[i] == b'-' {
                    i += 1;
                }

                // int part
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }

                // optional fractional part
                if i < bytes.len() && bytes[i] == b'.' {
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                }

                // optional %
                if i < bytes.len() && bytes[i] == b'%' {
                    i += 1;
                }

                if i == start {
                    return Err(InvalidToken);
                }

                let lexeme = &input[start..i];
                tokens.push(CssColorToken::Number(lexeme));
            }

            // single chars
            b'/' => {
                tokens.push(CssColorToken::Slash);
                i += 1;
            }
            b',' => {
                tokens.push(CssColorToken::Comma);
                i += 1;
            }

            // invalid
            _ => return Err(InvalidToken),
        }
    }

    Ok(tokens)
}

fn parse_rgb_component(num: &str) -> Result<u8, ColorParseError> {
    use ColorParseError::{InvalidToken, OutOfRange};

    let (core, is_percent) = if let Some(stripped) = num.strip_suffix('%') {
        (stripped, true)
    } else {
        (num, false)
    };

    if is_percent {
        let v: ColorFloat = core.parse().map_err(|_| InvalidToken)?;
        if !(0.0..=100.0).contains(&v) {
            return Err(OutOfRange);
        }
        let scaled = (v / 100.0 * 255.0).round();
        Ok(scaled.clamp(0.0, 255.0) as u8)
    } else {
        // try int first
        if let Ok(v_int) = core.parse::<u16>() {
            if v_int > 255 {
                return Err(OutOfRange);
            }
            Ok(v_int as u8)
        } else {
            let v: ColorFloat = core.parse().map_err(|_| InvalidToken)?;
            if !(0.0..=255.0).contains(&v) {
                return Err(OutOfRange);
            }
            Ok(v.round().clamp(0.0, 255.0) as u8)
        }
    }
}

fn parse_alpha_component(num: &str) -> Result<u8, ColorParseError> {
    use ColorParseError::{InvalidToken, OutOfRange};

    let (core, is_percent) = if let Some(stripped) = num.strip_suffix('%') {
        (stripped, true)
    } else {
        (num, false)
    };

    if is_percent {
        let v: ColorFloat = core.parse().map_err(|_| InvalidToken)?;
        if !(0.0..=100.0).contains(&v) {
            return Err(OutOfRange);
        }
        let scaled = (v / 100.0 * 255.0).round();
        Ok(scaled.clamp(0.0, 255.0) as u8)
    } else {
        // try 0.0..1.0 first
        if let Ok(vf) = core.parse::<ColorFloat>() {
            if (0.0..=1.0).contains(&vf) {
                let scaled = (vf * 255.0).round();
                return Ok(scaled.clamp(0.0, 255.0) as u8);
            }
        }

        // then try 0..255
        let vi: u16 = core.parse().map_err(|_| InvalidToken)?;
        if vi > 255 {
            return Err(OutOfRange);
        }
        Ok(vi as u8)
    }
}

/// Parse a hex color from a string.
///
/// The allowed formats are:
/// * #RGB
/// * #RGBA
/// * #RRGGBB
/// * #RRGGBBAA
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

/// Parse a CSS rgb function.
///
/// The allowed styles are:
/// rgb(r,g,b)
/// rgb(r g b)
/// rgb(r% g% b%)
/// rgb(r g b / a)
fn parse_css_rgb(args: &str) -> Result<Color, ColorParseError> {
    use ColorParseError::*;

    let tokens = tokenize(args)?;
    if tokens.is_empty() {
        return Err(InvalidFunc);
    }

    let has_comma = tokens.iter().any(|t| matches!(t, CssColorToken::Comma));

    let mut it = tokens.iter().peekable();
    let mut comps: Vec<&str> = Vec::new();
    let mut alpha: Option<&str> = None;

    if has_comma {
        // legacy rgb(r, g, b) or rgb(r, g, b, a)
        loop {
            let num = match it.next() {
                Some(CssColorToken::Number(s)) => *s,
                Some(_) => return Err(InvalidFunc),
                None => break,
            };
            comps.push(num);

            match it.next() {
                Some(CssColorToken::Comma) => continue,
                Some(CssColorToken::Slash) => {
                    // css technically doesn't have slash
                    // in the comma form, but it's easy
                    // enough to support.
                    let next = match it.next() {
                        Some(CssColorToken::Number(s)) => *s,
                        _ => return Err(InvalidToken),
                    };
                    alpha = Some(next);
                    if it.next().is_some() {
                        return Err(InvalidFunc);
                    }
                    break;
                }
                None => break,
                Some(_) => return Err(InvalidToken),
            }
        }
    } else {
        // modern: rgb(r g b / a?)
        while let Some(token) = it.next() {
            match token {
                CssColorToken::Number(s) => {
                    if alpha.is_some() {
                        return Err(InvalidFunc); // nums after alpha not allowed
                    }
                    comps.push(*s);
                }
                CssColorToken::Slash => {
                    if alpha.is_some() {
                        return Err(InvalidFunc); // double slash
                    }
                    let next = match it.next() {
                        Some(CssColorToken::Number(s)) => *s,
                        _ => return Err(InvalidToken),
                    };
                    alpha = Some(next);
                }
                CssColorToken::Comma => return Err(InvalidToken), // commas not allowed in this form
            }
        }
    }

    // need exactly 3 colors
    if comps.len() != 3 {
        return Err(InvalidFunc);
    }

    let r = parse_rgb_component(comps[0])?;
    let g = parse_rgb_component(comps[1])?;
    let b = parse_rgb_component(comps[2])?;
    let a = alpha.map(parse_alpha_component).transpose()?.unwrap_or(255);

    Ok(Color::from_rgba([r, g, b, a]))
}

/// Parse a CSS rgba function.
///
/// The allowed styles are:
/// rgba(r,g,b,a)
/// rgba(r g b a) (TODO: needs implementation)
/// rgba(r% g% b% a%) (TODO: needs implementation)
/// rgba(r g b / a) (TODO: needs implementation)
fn parse_css_rgba(args: &str) -> Result<Color, ColorParseError> {
    use ColorParseError::*;

    let nums: Vec<&str> = args.split(',').map(|t| t.trim()).collect();
    if nums.len() != 4 {
        return Err(InvalidFunc);
    }

    let r = nums[0]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;
    let g = nums[1]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;
    let b = nums[2]
        .parse::<u16>()
        .ok()
        .filter(|&v| v <= 255)
        .ok_or(OutOfRange)? as u8;

    // allow 0.0..1.0 or 0..255
    let a = if let Ok(f) = nums[3].parse::<f32>() {
        (f.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
    } else {
        nums[3]
            .parse::<u16>()
            .ok()
            .filter(|&v| v <= 255)
            .ok_or(OutOfRange)? as u8
    };

    Ok(Color::from_rgba([r, g, b, a]))
}

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

    // CSS-like: rgb(r,g,b) / rgba(r,g,b,a[0..1])
    // TODO: Parsing support is “CSS2-ish” right now
    // We only handle rgb(r,g,b) and rgba(r,g,b,a) with integers and commas. Modern CSS allows:
    // Space-separated: rgb(255 0 0)
    // Percentages: rgb(100% 0% 0%)
    // Slash alpha: rgb(255 0 0 / 0.5)
    // HSL: hsl(210 50% 40% / 0.7)
    // Plan: add a small tokenizer that accepts commas or spaces, and an optional / alpha token.
    // We already have HSL converters, so once the parser extracts (h, s%, l%, a?), we can call from_hsl.
    // Also allow for things like rgb(+255, +255, +255) (CSS allowed)
    let lower = s.to_ascii_lowercase();
    if let Some(args) = lower.strip_prefix("rgb(").and_then(|x| x.strip_suffix(')')) {
        return parse_css_rgb(args);
    }
    if let Some(args) = lower
        .strip_prefix("rgba(")
        .and_then(|x| x.strip_suffix(')'))
    {
        return parse_css_rgba(args);
    }

    Err(InvalidFunc)
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
