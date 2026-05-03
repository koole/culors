//! CSS functional-notation parser.
//!
//! Mirrors culori 4.0.2's tokenizer + `parseModernSyntax`
//! (`node_modules/culori/src/parse.js`) and the per-space `parse*.js`
//! files. The tokenizer is a hand-rolled port of the small CSS-syntax
//! subset culori implements (numbers with optional sign/exponent/unit,
//! percentages, hue units `deg`/`rad`/`grad`/`turn`, identifiers,
//! `none`, parentheses, slashes for alpha, commas).
//!
//! Function names are case-sensitive in culori; we mirror that. Legacy
//! comma-separated forms are handled by re-tokenizing the comma-stripped
//! string into the same modern token stream — culori's
//! `parseRgbLegacy` / `parseHslLegacy` use bespoke regexes, but the
//! functional outcome is the same: comma-separated `<number>` /
//! `<percentage>` triples plus an optional fourth alpha component.
//!
//! `none` channels become `f64::NAN` for the corresponding field; alpha
//! `none` becomes `alpha: None`, matching culori's behavior of omitting
//! the field entirely.

#![allow(dead_code)] // wired up by parse::mod.rs in a later commit

use crate::color::Color;
use crate::spaces::{Hsl, Hwb, Rgb};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Tok {
    /// Function name + opening paren consumed.
    Function,
    /// Bare identifier (not followed by `(`).
    Ident,
    /// Bare number.
    Number,
    /// `<number>%`.
    Percentage,
    /// `<number><deg|rad|grad|turn>` — converted to degrees on parse.
    Hue,
    /// `none` keyword (channel-position only).
    None,
    /// Closing `)`.
    ParenClose,
    /// `/` token followed by an alpha value (boxed inside the alpha).
    Alpha,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    pub kind: Tok,
    pub value: f64,
    pub ident: String,
    /// For `Alpha` tokens, the inner kind: Number, Percentage, or None.
    pub alpha_inner: Option<Box<Token>>,
}

impl Token {
    fn function(name: String) -> Self {
        Self {
            kind: Tok::Function,
            value: 0.0,
            ident: name,
            alpha_inner: None,
        }
    }
    fn ident(name: String) -> Self {
        Self {
            kind: Tok::Ident,
            value: 0.0,
            ident: name,
            alpha_inner: None,
        }
    }
    fn none() -> Self {
        Self {
            kind: Tok::None,
            value: 0.0,
            ident: String::new(),
            alpha_inner: None,
        }
    }
    fn paren_close() -> Self {
        Self {
            kind: Tok::ParenClose,
            value: 0.0,
            ident: String::new(),
            alpha_inner: None,
        }
    }
    fn number(value: f64) -> Self {
        Self {
            kind: Tok::Number,
            value,
            ident: String::new(),
            alpha_inner: None,
        }
    }
    fn percentage(value: f64) -> Self {
        Self {
            kind: Tok::Percentage,
            value,
            ident: String::new(),
            alpha_inner: None,
        }
    }
    fn hue(value: f64) -> Self {
        Self {
            kind: Tok::Hue,
            value,
            ident: String::new(),
            alpha_inner: None,
        }
    }
}

struct Cursor<'a> {
    chars: &'a [u8],
    i: usize,
}

impl<'a> Cursor<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            chars: s.as_bytes(),
            i: 0,
        }
    }

    fn peek(&self, offset: usize) -> Option<u8> {
        self.chars.get(self.i + offset).copied()
    }

    fn at_eof(&self) -> bool {
        self.i >= self.chars.len()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek(0) {
            if c == b' ' || c == b'\t' || c == b'\n' {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    fn is_digit(b: u8) -> bool {
        b.is_ascii_digit()
    }

    fn is_ident_start(b: u8) -> bool {
        b.is_ascii_alphabetic() || b == b'_' || b >= 0x80
    }

    fn is_ident_continue(b: u8) -> bool {
        Self::is_ident_start(b) || b.is_ascii_digit() || b == b'-'
    }

    /// Mirrors culori's `is_num`.
    fn at_num(&self) -> bool {
        let Some(c) = self.peek(0) else {
            return false;
        };
        if c == b'-' || c == b'+' {
            let Some(c1) = self.peek(1) else {
                return false;
            };
            if Self::is_digit(c1) {
                return true;
            }
            if c1 == b'.' {
                if let Some(c2) = self.peek(2) {
                    return Self::is_digit(c2);
                }
            }
            return false;
        }
        if c == b'.' {
            return self.peek(1).is_some_and(Self::is_digit);
        }
        Self::is_digit(c)
    }

    fn at_ident(&self) -> bool {
        let Some(c) = self.peek(0) else {
            return false;
        };
        if Self::is_ident_start(c) {
            return true;
        }
        if c == b'-' {
            let Some(c1) = self.peek(1) else {
                return false;
            };
            if c1 == b'-' || Self::is_ident_start(c1) {
                return true;
            }
        }
        false
    }

    fn read_digits(&mut self) -> String {
        let start = self.i;
        while let Some(c) = self.peek(0) {
            if Self::is_digit(c) {
                self.i += 1;
            } else {
                break;
            }
        }
        std::str::from_utf8(&self.chars[start..self.i])
            .unwrap_or("")
            .to_string()
    }

    /// Mirrors culori's `num`. Returns Some(token) where token is
    /// Number / Percentage / Hue, or None on parse failure.
    fn read_num(&mut self) -> Option<Token> {
        let mut s = String::new();
        if let Some(c) = self.peek(0) {
            if c == b'-' || c == b'+' {
                s.push(c as char);
                self.i += 1;
            }
        }
        s.push_str(&self.read_digits());
        if self.peek(0) == Some(b'.') && self.peek(1).is_some_and(Self::is_digit) {
            s.push('.');
            self.i += 1;
            s.push_str(&self.read_digits());
        }
        if matches!(self.peek(0), Some(b'e') | Some(b'E')) {
            let next = self.peek(1);
            if matches!(next, Some(b'-') | Some(b'+')) && self.peek(2).is_some_and(Self::is_digit) {
                s.push(self.peek(0).unwrap() as char);
                self.i += 1;
                s.push(self.peek(0).unwrap() as char);
                self.i += 1;
                s.push_str(&self.read_digits());
            } else if next.is_some_and(Self::is_digit) {
                s.push(self.peek(0).unwrap() as char);
                self.i += 1;
                s.push_str(&self.read_digits());
            }
        }
        let value: f64 = s.parse().ok()?;
        if self.at_ident() {
            let id = self.read_ident();
            return match id.as_str() {
                "deg" => Some(Token::hue(value)),
                "rad" => Some(Token::hue(value * (180.0 / std::f64::consts::PI))),
                "grad" => Some(Token::hue(value * 0.9)),
                "turn" => Some(Token::hue(value * 360.0)),
                _ => None,
            };
        }
        if self.peek(0) == Some(b'%') {
            self.i += 1;
            return Some(Token::percentage(value));
        }
        Some(Token::number(value))
    }

    fn read_ident(&mut self) -> String {
        let start = self.i;
        while let Some(c) = self.peek(0) {
            if Self::is_ident_continue(c) {
                self.i += 1;
            } else {
                break;
            }
        }
        std::str::from_utf8(&self.chars[start..self.i])
            .unwrap_or("")
            .to_string()
    }
}

pub(crate) fn tokenize(input: &str) -> Option<Vec<Token>> {
    let trimmed = input.trim();
    let mut cur = Cursor::new(trimmed);
    let mut tokens = Vec::new();
    while !cur.at_eof() {
        let c = cur.chars[cur.i];
        cur.i += 1;
        if c == b' ' || c == b'\t' || c == b'\n' {
            cur.skip_ws();
            continue;
        }
        if c == b',' {
            return None;
        }
        if c == b')' {
            tokens.push(Token::paren_close());
            continue;
        }
        if c == b'+' {
            cur.i -= 1;
            if cur.at_num() {
                tokens.push(cur.read_num()?);
                continue;
            }
            return None;
        }
        if c == b'-' {
            cur.i -= 1;
            if cur.at_num() {
                tokens.push(cur.read_num()?);
                continue;
            }
            if cur.at_ident() {
                tokens.push(Token::ident(cur.read_ident()));
                continue;
            }
            return None;
        }
        if c == b'.' {
            cur.i -= 1;
            if cur.at_num() {
                tokens.push(cur.read_num()?);
                continue;
            }
            return None;
        }
        if c == b'/' {
            cur.skip_ws();
            if cur.at_num() {
                let inner = cur.read_num()?;
                if matches!(inner.kind, Tok::Hue) {
                    return None;
                }
                tokens.push(Token {
                    kind: Tok::Alpha,
                    value: 0.0,
                    ident: String::new(),
                    alpha_inner: Some(Box::new(inner)),
                });
                continue;
            }
            if cur.at_ident() {
                let id = cur.read_ident();
                if id == "none" {
                    tokens.push(Token {
                        kind: Tok::Alpha,
                        value: 0.0,
                        ident: String::new(),
                        alpha_inner: Some(Box::new(Token::none())),
                    });
                    continue;
                }
            }
            return None;
        }
        if Cursor::is_digit(c) {
            cur.i -= 1;
            tokens.push(cur.read_num()?);
            continue;
        }
        if Cursor::is_ident_start(c) {
            cur.i -= 1;
            let id = cur.read_ident();
            if cur.peek(0) == Some(b'(') {
                cur.i += 1;
                tokens.push(Token::function(id));
            } else if id == "none" {
                tokens.push(Token::none());
            } else {
                tokens.push(Token::ident(id));
            }
            continue;
        }
        return None;
    }
    Some(tokens)
}

/// Modern-syntax parsed payload: `[function_name, c1, c2, c3, alpha]`.
/// Alpha is always present, possibly as a `Tok::None` placeholder.
pub(crate) struct Modern {
    pub func: String,
    pub coords: [Token; 4],
}

pub(crate) fn parse_modern(tokens: &[Token], include_hue: bool) -> Option<Modern> {
    parse_form(tokens, include_hue, false)
}

fn parse_form(tokens: &[Token], include_hue: bool, legacy: bool) -> Option<Modern> {
    let mut iter = tokens.iter();
    let first = iter.next()?;
    if first.kind != Tok::Function {
        return None;
    }
    let func = first.ident.clone();
    let coords = consume_coords(&mut iter, include_hue, legacy)?;
    Some(Modern { func, coords })
}

fn consume_coords<'a>(
    iter: &mut std::slice::Iter<'a, Token>,
    include_hue: bool,
    legacy_alpha_as_value: bool,
) -> Option<[Token; 4]> {
    let mut coords: Vec<Token> = Vec::new();
    let mut closed = false;
    for token in iter.by_ref() {
        if closed {
            return None;
        }
        match token.kind {
            Tok::None | Tok::Number | Tok::Percentage | Tok::Alpha => {
                coords.push(token.clone());
            }
            Tok::Hue if include_hue => coords.push(token.clone()),
            Tok::ParenClose => {
                closed = true;
            }
            _ => return None,
        }
    }
    if !closed {
        return None;
    }
    if coords.len() < 3 || coords.len() > 4 {
        return None;
    }
    let alpha_token: Token = if coords.len() == 4 {
        let last = coords.pop().unwrap();
        if last.kind == Tok::Alpha {
            *last.alpha_inner.unwrap()
        } else if legacy_alpha_as_value
            && matches!(last.kind, Tok::Number | Tok::Percentage | Tok::None)
        {
            // legacy comma-form: the 4th argument is the alpha value
            // directly, with no `/` separator.
            last
        } else {
            return None;
        }
    } else {
        Token::none()
    };
    if coords.iter().any(|c| c.kind == Tok::Alpha) {
        return None;
    }
    let c1 = coords.remove(0);
    let c2 = coords.remove(0);
    let c3 = coords.remove(0);
    Some([c1, c2, c3, alpha_token])
}

/// Strip commas to translate legacy comma-form into modern-form. Returns
/// the cleaned string only when the input starts with a known legacy
/// function (rgb/rgba/hsl/hsla); otherwise returns None to defer to the
/// modern-form parse. This is a behavioral shortcut: culori's actual
/// pipeline is two distinct regex parsers, but the legacy regexes only
/// accept tokens already accepted by the modern parser plus commas, so
/// removing commas and rerunning the modern parser produces the same
/// result.
fn strip_legacy_commas(input: &str) -> Option<String> {
    let trimmed = input.trim_start();
    let prefix = ["rgba(", "rgb(", "hsla(", "hsl("]
        .iter()
        .find(|p| trimmed.starts_with(*p))?;
    let _ = prefix;
    Some(input.replace(',', " "))
}

fn alpha_value(t: &Token) -> Option<f64> {
    match t.kind {
        Tok::None => None,
        Tok::Number => Some(t.value.clamp(0.0, 1.0)),
        Tok::Percentage => Some((t.value / 100.0).clamp(0.0, 1.0)),
        _ => None,
    }
}

/// Parse `rgb()` / `rgba()` modern or legacy form. Returns `None` if
/// `parsed` doesn't belong to this function family.
fn parse_rgb(parsed: &Modern) -> Option<Rgb> {
    if parsed.func != "rgb" && parsed.func != "rgba" {
        return None;
    }
    let [r, g, b, a] = &parsed.coords;
    if matches!(r.kind, Tok::Hue) || matches!(g.kind, Tok::Hue) || matches!(b.kind, Tok::Hue) {
        return None;
    }
    let resolve = |t: &Token| match t.kind {
        Tok::None => f64::NAN,
        Tok::Number => t.value / 255.0,
        Tok::Percentage => t.value / 100.0,
        _ => f64::NAN,
    };
    Some(Rgb {
        r: resolve(r),
        g: resolve(g),
        b: resolve(b),
        alpha: alpha_value(a),
    })
}

fn parse_hsl(parsed: &Modern) -> Option<Hsl> {
    if parsed.func != "hsl" && parsed.func != "hsla" {
        return None;
    }
    let [h, s, l, a] = &parsed.coords;
    // Hue: number or angle; reject percentage.
    let h_val = match h.kind {
        Tok::None => f64::NAN,
        Tok::Number | Tok::Hue => h.value,
        Tok::Percentage => return None,
        _ => return None,
    };
    // S / L: percentage in spec; culori also accepts number-as-percentage.
    let s_val = match s.kind {
        Tok::None => f64::NAN,
        Tok::Hue => return None,
        Tok::Number | Tok::Percentage => s.value / 100.0,
        _ => return None,
    };
    let l_val = match l.kind {
        Tok::None => f64::NAN,
        Tok::Hue => return None,
        Tok::Number | Tok::Percentage => l.value / 100.0,
        _ => return None,
    };
    Some(Hsl {
        h: h_val,
        s: s_val,
        l: l_val,
        alpha: alpha_value(a),
    })
}

fn parse_hwb(parsed: &Modern) -> Option<Hwb> {
    if parsed.func != "hwb" {
        return None;
    }
    let [h, w, b, a] = &parsed.coords;
    let h_val = match h.kind {
        Tok::None => f64::NAN,
        Tok::Number | Tok::Hue => h.value,
        Tok::Percentage => return None,
        _ => return None,
    };
    let w_val = match w.kind {
        Tok::None => f64::NAN,
        Tok::Hue => return None,
        Tok::Number | Tok::Percentage => w.value / 100.0,
        _ => return None,
    };
    let b_val = match b.kind {
        Tok::None => f64::NAN,
        Tok::Hue => return None,
        Tok::Number | Tok::Percentage => b.value / 100.0,
        _ => return None,
    };
    Some(Hwb {
        h: h_val,
        w: w_val,
        b: b_val,
        alpha: alpha_value(a),
    })
}

/// Try every functional notation handled by this module. Returns
/// `Some(Color)` on a recognized + valid call, `None` on either a
/// non-functional input or a malformed call. Callers above this layer
/// disambiguate by trying named/hex first.
pub(crate) fn parse_functional(input: &str) -> Option<Color> {
    // Try modern-syntax tokenization first.
    if let Some(tokens) = tokenize(input) {
        if let Some(parsed) = parse_modern(&tokens, true) {
            if let Some(c) = parse_rgb(&parsed) {
                return Some(Color::Rgb(c));
            }
            if let Some(c) = parse_hsl(&parsed) {
                return Some(Color::Hsl(c));
            }
            if let Some(c) = parse_hwb(&parsed) {
                return Some(Color::Hwb(c));
            }
        }
    }
    // Legacy comma-form: convert commas to whitespace, retry, treating
    // a 4th positional argument as the alpha value.
    if let Some(stripped) = strip_legacy_commas(input) {
        if let Some(tokens) = tokenize(&stripped) {
            if let Some(parsed) = parse_form(&tokens, true, true) {
                if let Some(c) = parse_rgb(&parsed) {
                    return Some(Color::Rgb(c));
                }
                if let Some(c) = parse_hsl(&parsed) {
                    return Some(Color::Hsl(c));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rgb(actual: Color, r: f64, g: f64, b: f64, alpha: Option<f64>) {
        let Color::Rgb(c) = actual else {
            panic!("expected Rgb, got {actual:?}");
        };
        assert!(approx(c.r, r), "r: {} vs {}", c.r, r);
        assert!(approx(c.g, g), "g: {} vs {}", c.g, g);
        assert!(approx(c.b, b), "b: {} vs {}", c.b, b);
        assert_eq!(c.alpha, alpha);
    }

    fn approx(a: f64, b: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        (a - b).abs() < 1e-12
    }

    #[test]
    fn rgb_modern_numbers() {
        assert_rgb(
            parse_functional("rgb(255 0 0)").unwrap(),
            1.0,
            0.0,
            0.0,
            None,
        );
    }

    #[test]
    fn rgb_modern_with_alpha() {
        assert_rgb(
            parse_functional("rgb(255 0 0 / 0.5)").unwrap(),
            1.0,
            0.0,
            0.0,
            Some(0.5),
        );
    }

    #[test]
    fn rgb_modern_with_pct_alpha() {
        assert_rgb(
            parse_functional("rgb(255 0 0 / 50%)").unwrap(),
            1.0,
            0.0,
            0.0,
            Some(0.5),
        );
    }

    #[test]
    fn rgb_legacy() {
        assert_rgb(
            parse_functional("rgb(255, 0, 0)").unwrap(),
            1.0,
            0.0,
            0.0,
            None,
        );
        assert_rgb(
            parse_functional("rgba(255, 0, 0, 0.5)").unwrap(),
            1.0,
            0.0,
            0.0,
            Some(0.5),
        );
    }

    #[test]
    fn rgb_legacy_pct() {
        assert_rgb(
            parse_functional("rgb(100%, 0%, 0%)").unwrap(),
            1.0,
            0.0,
            0.0,
            None,
        );
        assert_rgb(
            parse_functional("rgb(50%, 0%, 0%, 50%)").unwrap(),
            0.5,
            0.0,
            0.0,
            Some(0.5),
        );
    }

    #[test]
    fn rgb_none_channel_becomes_nan() {
        let Color::Rgb(c) = parse_functional("rgb(none 0 0)").unwrap() else {
            panic!()
        };
        assert!(c.r.is_nan());
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.alpha, None);
    }

    #[test]
    fn rgb_none_alpha_keeps_alpha_none() {
        let Color::Rgb(c) = parse_functional("rgb(255 0 0 / none)").unwrap() else {
            panic!()
        };
        assert_eq!(c.r, 1.0);
        assert_eq!(c.alpha, None);
    }

    #[test]
    fn rgb_oor_passthrough() {
        // culori does not clamp rgb channel values.
        assert_rgb(
            parse_functional("rgb(300 0 0)").unwrap(),
            300.0 / 255.0,
            0.0,
            0.0,
            None,
        );
        assert_rgb(
            parse_functional("rgb(-10 0 0)").unwrap(),
            -10.0 / 255.0,
            0.0,
            0.0,
            None,
        );
    }

    #[test]
    fn rgb_too_few_args_fails() {
        assert!(parse_functional("rgb(255 0)").is_none());
        assert!(parse_functional("rgb(not enough)").is_none());
    }

    #[test]
    fn rgb_capitalized_function_fails() {
        assert!(parse_functional("RGB(255 0 0)").is_none());
        assert!(parse_functional("Rgb(255 0 0)").is_none());
    }

    #[test]
    fn rgb_hue_in_channel_fails() {
        assert!(parse_functional("rgb(120deg 0 0)").is_none());
    }

    #[test]
    fn hsl_modern() {
        let Color::Hsl(c) = parse_functional("hsl(120deg 50% 50%)").unwrap() else {
            panic!()
        };
        assert_eq!(c.h, 120.0);
        assert_eq!(c.s, 0.5);
        assert_eq!(c.l, 0.5);
        assert_eq!(c.alpha, None);
    }

    #[test]
    fn hsl_legacy() {
        let Color::Hsl(c) = parse_functional("hsl(120, 50%, 50%)").unwrap() else {
            panic!()
        };
        assert_eq!(c.h, 120.0);
        assert_eq!(c.s, 0.5);
        assert_eq!(c.l, 0.5);
    }

    #[test]
    fn hsl_turn_unit() {
        let Color::Hsl(c) = parse_functional("hsl(0.5turn 100% 50%)").unwrap() else {
            panic!()
        };
        assert_eq!(c.h, 180.0);
        assert_eq!(c.s, 1.0);
        assert_eq!(c.l, 0.5);
    }

    #[test]
    fn hsl_with_alpha() {
        let Color::Hsl(c) = parse_functional("hsl(120 50% 50% / 0.5)").unwrap() else {
            panic!()
        };
        assert_eq!(c.alpha, Some(0.5));
    }

    #[test]
    fn hsl_pct_hue_fails() {
        assert!(parse_functional("hsl(50% 50% 50%)").is_none());
    }

    #[test]
    fn hwb_modern() {
        let Color::Hwb(c) = parse_functional("hwb(120 30% 30%)").unwrap() else {
            panic!()
        };
        assert_eq!(c.h, 120.0);
        assert_eq!(c.w, 0.3);
        assert_eq!(c.b, 0.3);
        assert_eq!(c.alpha, None);
    }

    #[test]
    fn hwb_with_alpha_pct() {
        let Color::Hwb(c) = parse_functional("hwb(120 30% 30% / 50%)").unwrap() else {
            panic!()
        };
        assert_eq!(c.alpha, Some(0.5));
    }

    #[test]
    fn whitespace_inside_function_ok() {
        assert_rgb(
            parse_functional("rgb( 255 , 0 , 0 )").unwrap(),
            1.0,
            0.0,
            0.0,
            None,
        );
    }

    #[test]
    fn extra_token_after_paren_close_fails() {
        assert!(parse_functional("rgb(1 2 3)x").is_none());
    }
}
