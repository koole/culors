//! CSS hex color parser.
//!
//! Mirrors culori's `rgb/parseHex.js` + `parseNumber.js`. Accepts the four
//! lengths defined in CSS Color Module 4 — 3, 4, 6, 8 hex digits with an
//! optional leading `#`. The match is case-insensitive but rejects any
//! surrounding whitespace; culori's regex is anchored with `^...$`.
//!
//! 3- and 4-digit forms duplicate each nibble, so `#abc` becomes
//! `#aabbcc` and `#abcd` becomes `#aabbccdd`. The 6-digit form has no
//! alpha component (alpha stays `None`); the 8-digit form attaches an
//! explicit alpha including the `#0000` "transparent" case. Channels are
//! normalized to 0..1 in line with culori.

use crate::spaces::Rgb;

/// Parse a hex color literal. Returns `None` for any input that doesn't
/// fit one of the four hex lengths or contains non-hex characters.
pub(crate) fn parse_hex(input: &str) -> Option<Rgb> {
    let body = input.strip_prefix('#').unwrap_or(input);
    if !body.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let value = u64::from_str_radix(body, 16).ok()?;
    match body.len() {
        3 => {
            // 0xRGB -> 0xRRGGBB
            let r = (((value >> 8) & 0xf) | ((value >> 4) & 0xf0)) as f64 / 255.0;
            let g = (((value >> 4) & 0xf) | (value & 0xf0)) as f64 / 255.0;
            let b = ((value & 0xf) | ((value << 4) & 0xf0)) as f64 / 255.0;
            Some(Rgb {
                r,
                g,
                b,
                alpha: None,
            })
        }
        4 => {
            // 0xRGBA -> 0xRRGGBBAA
            let r = (((value >> 12) & 0xf) | ((value >> 8) & 0xf0)) as f64 / 255.0;
            let g = (((value >> 8) & 0xf) | ((value >> 4) & 0xf0)) as f64 / 255.0;
            let b = (((value >> 4) & 0xf) | (value & 0xf0)) as f64 / 255.0;
            let a = ((value & 0xf) | ((value << 4) & 0xf0)) as f64 / 255.0;
            Some(Rgb {
                r,
                g,
                b,
                alpha: Some(a),
            })
        }
        6 => Some(Rgb {
            r: ((value >> 16) & 0xff) as f64 / 255.0,
            g: ((value >> 8) & 0xff) as f64 / 255.0,
            b: (value & 0xff) as f64 / 255.0,
            alpha: None,
        }),
        8 => Some(Rgb {
            r: ((value >> 24) & 0xff) as f64 / 255.0,
            g: ((value >> 16) & 0xff) as f64 / 255.0,
            b: ((value >> 8) & 0xff) as f64 / 255.0,
            alpha: Some((value & 0xff) as f64 / 255.0),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(r: f64, g: f64, b: f64, alpha: Option<f64>) -> Rgb {
        Rgb { r, g, b, alpha }
    }

    #[test]
    fn three_digit() {
        assert_eq!(parse_hex("#f00"), Some(rgb(1.0, 0.0, 0.0, None)));
        // #abc -> #aabbcc -> r=170/255, g=187/255, b=204/255
        let expected = rgb(170.0 / 255.0, 187.0 / 255.0, 204.0 / 255.0, None);
        assert_eq!(parse_hex("#abc"), Some(expected));
    }

    #[test]
    fn four_digit() {
        // #0000 -> r=0, g=0, b=0, alpha=0 (transparent black via hex)
        assert_eq!(parse_hex("#0000"), Some(rgb(0.0, 0.0, 0.0, Some(0.0))));
        // #f00f -> opaque red
        assert_eq!(parse_hex("#f00f"), Some(rgb(1.0, 0.0, 0.0, Some(1.0))));
    }

    #[test]
    fn six_digit() {
        assert_eq!(parse_hex("#ff0000"), Some(rgb(1.0, 0.0, 0.0, None)));
        assert_eq!(parse_hex("#000000"), Some(rgb(0.0, 0.0, 0.0, None)));
    }

    #[test]
    fn eight_digit() {
        assert_eq!(parse_hex("#ff0000ff"), Some(rgb(1.0, 0.0, 0.0, Some(1.0))));
        assert_eq!(
            parse_hex("#ff000080"),
            Some(rgb(1.0, 0.0, 0.0, Some(128.0 / 255.0)))
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(parse_hex("#FF0000"), parse_hex("#ff0000"));
        assert_eq!(parse_hex("#FfA500"), parse_hex("#ffa500"));
    }

    #[test]
    fn leading_hash_optional() {
        // culori's regex makes `#` optional.
        assert_eq!(parse_hex("ff0000"), parse_hex("#ff0000"));
        assert_eq!(parse_hex("f00"), parse_hex("#f00"));
    }

    #[test]
    fn invalid_lengths_rejected() {
        assert_eq!(parse_hex("#"), None);
        assert_eq!(parse_hex("#1"), None);
        assert_eq!(parse_hex("#12"), None);
        assert_eq!(parse_hex("#12345"), None);
        assert_eq!(parse_hex("#1234567"), None);
        assert_eq!(parse_hex("#123456789"), None);
        assert_eq!(parse_hex(""), None);
    }

    #[test]
    fn non_hex_chars_rejected() {
        assert_eq!(parse_hex("#xyz"), None);
        assert_eq!(parse_hex("#gg0000"), None);
        assert_eq!(parse_hex("# f00"), None);
        assert_eq!(parse_hex("#f00 "), None);
    }
}
