// ==== src/arabic_reshaper/reordering.rs ====

use alloc::string::String;
use alloc::vec::Vec;
use core::iter::Peekable;

use super::detection::{is_arabic_char, is_bidi_mark, is_arabic_numeral};
use super::shaping::apply_arabic_shaping;
use super::normalize::normalize_visual_rtl_output;

/// Main entry point: Shapes Arabic letters into presentation forms
/// and performs visual right-to-left reordering suitable for LTR game engines
/// (Unity, RimWorld, etc.) while protecting tags, escapes, entities, and LTR words.
pub fn reshape_arabic(text: &str) -> String {
    if !super::detection::contains_arabic_letters(text) {
        return String::from(text);
    }
    reshape_preserving_newlines(text)
}

/// Handles line splitting while preserving the exact form of line separators
/// (literal "\\n" vs real '\n') — important for strings coming from game data files.
fn reshape_preserving_newlines(text: &str) -> String {
    if text.contains("\\n") {
        text.split("\\n").map(reshape_single_line).collect::<Vec<_>>().join("\\n")
    } else if text.contains('\n') {
        text.split('\n').map(reshape_single_line).collect::<Vec<_>>().join("\n")
    } else {
        reshape_single_line(text)
    }
}

fn reshape_single_line(line: &str) -> String {
    if line.is_empty() {
        return String::new();
    }
    let shaped = apply_arabic_shaping(line);
    let visual = manual_rtl_for_unity(&shaped);
    normalize_visual_rtl_output(&visual)
}

/// Core logic: tokenizes the string, classifies chunks, reverses token order per line
/// (logical RTL → visual LTR), reverses Arabic characters individually inside RTL runs,
/// and protects game-specific tags/placeholders and English/LTR words.
pub fn manual_rtl_for_unity(text: &str) -> String {
    #[derive(Debug, Clone, PartialEq)]
    enum Token {
        Tag(String),
        Ltr(String),
        Rtl(String),
        Neutral(String),
    }

    fn push_classified(buf: &mut String, tokens: &mut Vec<Token>) {
        if buf.is_empty() { return; }

        let has_ar = buf.chars().any(is_arabic_char);
        if has_ar {
            tokens.push(Token::Rtl(core::mem::take(buf)));
            return;
        }

        if buf.trim().is_empty() {
            tokens.push(Token::Neutral(core::mem::take(buf)));
            return;
        }

        let all_neutral = buf.chars().all(|c|
            c.is_whitespace() || is_bidi_mark(c) || (!c.is_alphanumeric() && !is_arabic_char(c))
        );

        if all_neutral {
            tokens.push(Token::Neutral(core::mem::take(buf)));
        } else {
            tokens.push(Token::Ltr(core::mem::take(buf)));
        }
    }

    fn read_balanced(chars: &mut Peekable<core::str::Chars<'_>>, open: char, close: char) -> String {
        let mut out = String::new();
        let mut depth = 0usize;
        while let Some(c) = chars.next() {
            out.push(c);
            if c == open { depth += 1; }
            else if c == close {
                depth = depth.saturating_sub(1);
                if depth == 0 { break; }
            }
        }
        out
    }

    fn read_xml_entity(chars: &mut Peekable<core::str::Chars<'_>>) -> Option<String> {
        let mut clone = chars.clone();
        if clone.next()? != '&' { return None; }

        let mut entity = String::from("&");
        let mut consumed = 1usize;

        while let Some(c) = clone.next() {
            consumed += 1;
            entity.push(c);
            if c == ';' {
                for _ in 0..consumed { let _ = chars.next(); }
                return Some(entity);
            }
            if consumed > 12 { break; }
            if !(c.is_ascii_alphanumeric() || matches!(c, '#' | 'x' | 'X' | ';')) { break; }
        }
        None
    }

    fn read_escape(chars: &mut Peekable<core::str::Chars<'_>>) -> Option<String> {
        let mut clone = chars.clone();
        if clone.next()? != '\\' { return None; }
        let next = clone.next()?;
        if matches!(next, 'n' | 't' | 'r' | '\\') {
            let mut out = String::new();
            out.push(chars.next().unwrap());
            out.push(chars.next().unwrap());
            return Some(out);
        }
        None
    }

    /// Reads a run of characters that should be kept in Left-to-Right order.
    /// Includes ASCII, standard numbers, and Arabic-Indic numerals.
    fn read_ltr_run(chars: &mut Peekable<core::str::Chars<'_>>) -> Option<String> {
        let &first = chars.peek()?;
        let is_ltr_start = first.is_ascii_alphanumeric()
            || is_arabic_numeral(first)
            || matches!(first, '"' | '\'' | '-' | '+' | '/' | '_' | '.' | ':' | '%');

        if !is_ltr_start {
            return None;
        }

        let mut out = String::new();
        while let Some(&c) = chars.peek() {
            if matches!(c, '{' | '}' | '<' | '>' | '[' | ']') { break; }
            if c.is_ascii() || c.is_whitespace() || is_arabic_numeral(c) {
                out.push(chars.next().unwrap());
            } else {
                break;
            }
        }
        if out.is_empty() { None } else { Some(out) }
    }

    /// Reads a "quoted ASCII span" like "-popupwindow" or "Steam" or "General" as a
    /// single atomic LTR token. This prevents the surrounding quotes from being
    /// split off and reversed into the Arabic stream.
    fn read_quoted_ltr(chars: &mut Peekable<core::str::Chars<'_>>) -> Option<String> {
        // 1. Validation phase (using a clone to peek ahead)
        {
            let mut clone = chars.clone();
            if clone.next()? != '"' {
                return None;
            }

            let mut inner_count = 0usize;
            let mut found_close = false;

            for ic in clone.by_ref() {
                if ic == '"' {
                    found_close = true;
                    break;
                }

                // Only treat as a protected LTR span if the content is pure ASCII.
                // If it contains Arabic/UTF8, we must return None so the
                // standard RTL logic can shape and reverse the text inside.
                if ic.is_ascii() {
                    inner_count += 1;
                } else {
                    return None;
                }
            }

            // Guard against unclosed quotes or empty quotes ("")
            if !found_close || inner_count == 0 {
                return None;
            }
        }

        // 2. Consumption phase (actually advancing the real iterator)
        chars.next(); // Consume opening quote
        let mut inner = String::new();
        loop {
            match chars.next() {
                Some('"') => break,
                Some(c)   => inner.push(c),
                None      => break,
            }
        }
        Some(alloc::format!("\"{inner}\""))
    }

    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = text.chars().peekable();
    let mut buf = String::new();

    while let Some(&c) = chars.peek() {
        // Balanced game tags: {…} <…> […]
        if matches!(c, '{' | '<' | '[') {
            push_classified(&mut buf, &mut tokens);
            let (open, close) = match c {
                '{' => ('{', '}'),
                '<' => ('<', '>'),
                '[' => ('[', ']'),
                _   => unreachable!(),
            };
            tokens.push(Token::Tag(read_balanced(&mut chars, open, close)));
            continue;
        }

        // XML entities: &quot; &amp; etc.
        if c == '&' {
            if let Some(entity) = read_xml_entity(&mut chars) {
                push_classified(&mut buf, &mut tokens);
                tokens.push(Token::Tag(entity));
                continue;
            }
        }

        // Backslash escapes: \n \t \r \\
        if c == '\\' {
            if let Some(esc) = read_escape(&mut chars) {
                push_classified(&mut buf, &mut tokens);
                tokens.push(Token::Tag(if esc == "\\n" { String::from("\n") } else { esc }));
                continue;
            }
        }

        // Real newline
        if c == '\n' {
            push_classified(&mut buf, &mut tokens);
            let _ = chars.next();
            tokens.push(Token::Tag(String::from("\n")));
            continue;
        }

        // Arabic letters and bidi marks go into the RTL buffer
        if is_arabic_char(c) || is_bidi_mark(c) {
            buf.push(chars.next().unwrap());
            continue;
        }

        // Quoted ASCII spans: "-popupwindow" / "Steam" / "General" → atomic Ltr token
        if c == '"' {
            if let Some(quoted) = read_quoted_ltr(&mut chars) {
                push_classified(&mut buf, &mut tokens);
                tokens.push(Token::Ltr(quoted));
                continue;
            }
        }

        // Plain LTR runs: RimWorld, Steam, numerals, key combos, etc.
        if let Some(ltr) = read_ltr_run(&mut chars) {
            push_classified(&mut buf, &mut tokens);
            tokens.push(Token::Ltr(ltr));
            continue;
        }

        // Whitespace and punctuation → neutral
        if c.is_whitespace() || c.is_ascii_punctuation() {
            push_classified(&mut buf, &mut tokens);
            let mut neutral = String::new();
            while let Some(&nc) = chars.peek() {
                if matches!(nc, '{' | '<' | '[' | '&' | '\\' | '"') { break; }
                if nc.is_whitespace() || nc.is_ascii_punctuation() {
                    neutral.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(Token::Neutral(neutral));
            continue;
        }

        buf.push(chars.next().unwrap());
    }

    push_classified(&mut buf, &mut tokens);

    // Split into lines at newline tokens and reverse token order per line
    let mut lines: Vec<Vec<Token>> = Vec::new();
    let mut current_line: Vec<Token> = Vec::new();

    for token in tokens {
        if token == Token::Tag(String::from("\n")) {
            current_line.reverse();
            lines.push(current_line);
            current_line = Vec::new();
        } else {
            current_line.push(token);
        }
    }
    current_line.reverse();
    lines.push(current_line);

    // Build final string
    let mut result = String::new();
    for (i, line) in lines.into_iter().enumerate() {
        if i > 0 { result.push('\n'); }

        let mut prev_kind: Option<&'static str> = None;

        for token in line {
            let (kind, rendered) = match token {
                Token::Rtl(s)     => ("rtl", s.chars().rev().collect::<String>()),
                Token::Ltr(s)     => ("ltr", s.clone()),
                Token::Tag(s)     => ("tag", s.clone()),
                Token::Neutral(s) => ("neutral", s.clone()),
            };

            if let Some(pk) = prev_kind {
                let crossing = (pk == "rtl" && (kind == "ltr" || kind == "tag"))
                            || ((pk == "ltr" || pk == "tag") && kind == "rtl");

                if crossing
                    && !result.ends_with(char::is_whitespace)
                    && !rendered.starts_with(char::is_whitespace)
                {
                    let first = rendered.chars().next();
                    let looks_puncty = matches!(first, Some(c)
                        if c.is_ascii_punctuation() && c != '"' && c != '\''
                    );
                    if !looks_puncty {
                        result.push(' ');
                    }
                }
            }

            result.push_str(&rendered);
            prev_kind = Some(kind);
        }
    }

    result
}
