//! Small helpers that reproduce Python string semantics the reference build relied on.

use crate::unicode;

/// Python `str.strip()` (Unicode whitespace as defined by `str.isspace`).
pub fn py_strip(s: &str) -> &str {
    s.trim_matches(unicode::is_space)
}

/// Python `str.split()` with no arguments: split on runs of whitespace, drop empties.
pub fn py_split_ws(s: &str) -> Vec<&str> {
    s.split(unicode::is_space).filter(|p| !p.is_empty()).collect()
}

/// Python `repr()` of a `str`.
pub fn py_repr_str(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if is_py_printable(c) => out.push(c),
            c => {
                let cp = c as u32;
                if cp < 0x100 {
                    out.push_str(&format!("\\x{:02x}", cp));
                } else if cp < 0x10000 {
                    out.push_str(&format!("\\u{:04x}", cp));
                } else {
                    out.push_str(&format!("\\U{:08x}", cp));
                }
            }
        }
    }
    out.push(quote);
    out
}

/// Approximation of Python's `str.isprintable()` for repr purposes: control
/// characters, separators other than space, and unassigned/format characters
/// are escaped. Only used in error messages.
fn is_py_printable(c: char) -> bool {
    let cp = c as u32;
    if c == ' ' {
        return true;
    }
    if cp < 0x20 || (0x7f..=0xa0).contains(&cp) || cp == 0xad {
        return false;
    }
    if unicode::is_space(c) {
        return false;
    }
    !matches!(cp, 0x200b..=0x200f | 0x2028..=0x202e | 0x2060..=0x206f | 0xfeff | 0xfff9..=0xfffb)
        && !(0xd800..=0xdfff).contains(&cp)
}

/// Jinja2/MarkupSafe autoescape.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&#34;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// markdown-it `escapeHtml` (does not escape `'`).
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

/// Collect a char slice into a `String`.
pub fn chars_to_string(c: &[char]) -> String {
    c.iter().collect()
}
