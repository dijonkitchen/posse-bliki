//! Front-matter: splitting, a small YAML subset, and validation equivalent to
//! `spec/content-schema.json` (Draft 2020-12, as the reference build ran it).
//!
//! Supported YAML: a block mapping of `key: value` lines whose values are plain
//! scalars (resolved with YAML 1.1 rules, minus timestamps — dates stay strings),
//! single/double-quoted strings, literal/folded block scalars, flow sequences
//! (`[a, "b"]`) and block sequences (`- a`) of scalars. Anything else is
//! reported as an error rather than guessed at.

use crate::util::{py_repr_str, py_strip};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
}

impl Value {
    /// Python `repr()` of the equivalent object (used in error messages).
    pub fn repr(&self) -> String {
        match self {
            Value::Null => "None".into(),
            Value::Bool(b) => if *b { "True" } else { "False" }.into(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => py_float_repr(*f),
            Value::Str(s) => py_repr_str(s),
            Value::List(l) => format!("[{}]", l.iter().map(|v| v.repr()).collect::<Vec<_>>().join(", ")),
            Value::Map(m) => format!(
                "{{{}}}",
                m.iter().map(|(k, v)| format!("{}: {}", py_repr_str(k), v.repr())).collect::<Vec<_>>().join(", ")
            ),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Python truthiness.
    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
        }
    }
}

fn py_float_repr(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let s = format!("{:?}", f);
    // Rust prints 1e20 as "1e20"; Python as "1e+20".
    if let Some(idx) = s.find('e') {
        let (m, e) = s.split_at(idx);
        let e = &e[1..];
        let (sign, digits) = if let Some(d) = e.strip_prefix('-') { ("-", d) } else { ("+", e) };
        return format!("{}e{}{:0>2}", m.trim_end_matches(".0"), sign, digits);
    }
    s
}

/// Ordered string-keyed mapping parsed from front-matter.
pub type FrontMatter = Vec<(String, Value)>;

pub fn get<'a>(fm: &'a FrontMatter, key: &str) -> Option<&'a Value> {
    fm.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// `\A---\n(.+?)\n---\n?(.*)\Z` with DOTALL. Returns `(yaml, body)`.
pub fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    // group 1 needs at least one character, so search from the 2nd char of rest.
    let mut first = rest.chars();
    let first_len = first.next()?.len_utf8();
    let idx = rest[first_len..].find("\n---")? + first_len;
    let yaml = &rest[..idx];
    let mut body = &rest[idx + 4..];
    if let Some(b) = body.strip_prefix('\n') {
        body = b;
    }
    Some((yaml, body))
}

// ---------------------------------------------------------------------------
// YAML subset
// ---------------------------------------------------------------------------

struct Line<'a> {
    no: usize,
    indent: usize,
    text: &'a str, // without indentation
}

fn yerr<T>(no: usize, msg: &str) -> Result<T, String> {
    Err(format!("YAML error on front-matter line {}: {}", no + 1, msg))
}

/// Parse the YAML document. `Ok(None)` is an empty document (`null`).
pub fn parse_yaml(src: &str) -> Result<Option<Value>, String> {
    let mut lines: Vec<Line> = Vec::new();
    for (no, raw) in src.split('\n').enumerate() {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if raw.contains('\t') && raw.trim_start_matches(' ').starts_with('\t') {
            return yerr(no, "tabs are not allowed for indentation");
        }
        let indent = raw.len() - raw.trim_start_matches(' ').len();
        lines.push(Line { no, indent, text: &raw[indent..] });
    }
    let is_blank = |l: &Line| {
        let t = l.text.trim_end();
        t.is_empty() || t.starts_with('#')
    };
    let mut i = 0;
    while i < lines.len() && is_blank(&lines[i]) {
        i += 1;
    }
    if i >= lines.len() {
        return Ok(None);
    }
    let first = &lines[i];
    if first.text == "---" || first.text.starts_with("--- ") || first.text.starts_with("%") {
        return yerr(first.no, "multiple documents and directives are not supported");
    }
    if first.text == "-" || first.text.starts_with("- ") {
        // top-level sequence: not a mapping
        let (v, _) = parse_block_seq(&lines, i, first.indent)?;
        return Ok(Some(v));
    }
    if find_mapping_colon(first.text).is_none() {
        // a lone scalar document
        let (v, _) = parse_value_text(&lines, i, first.text, first.indent, true)?;
        return Ok(Some(v));
    }
    let base = first.indent;
    let mut map: Vec<(String, Value)> = Vec::new();
    while i < lines.len() {
        if is_blank(&lines[i]) {
            i += 1;
            continue;
        }
        let l = &lines[i];
        if l.indent != base {
            return yerr(l.no, "unexpected indentation");
        }
        let colon = match find_mapping_colon(l.text) {
            Some(c) => c,
            None => return yerr(l.no, "expected a `key: value` line"),
        };
        let key_raw = l.text[..colon].trim_end();
        let key = match parse_key(key_raw) {
            Some(k) => k,
            None => return yerr(l.no, "unsupported mapping key"),
        };
        let rest = &l.text[colon + 1..];
        let (value, next) = parse_value_text(&lines, i, rest, base, false)?;
        if let Some(slot) = map.iter_mut().find(|(k, _)| *k == key) {
            slot.1 = value;
        } else {
            map.push((key, value));
        }
        i = next;
    }
    Ok(Some(Value::Map(map)))
}

fn parse_key(k: &str) -> Option<String> {
    if k.is_empty() {
        return None;
    }
    if k.starts_with('"') || k.starts_with('\'') {
        let chars: Vec<char> = k.chars().collect();
        let (s, end) = parse_quoted(&chars, 0).ok()?;
        if end != chars.len() {
            return None;
        }
        return Some(s);
    }
    if k.starts_with(['?', '&', '*', '!', '[', '{', '|', '>', '@', '`', '%']) {
        return None;
    }
    Some(k.to_string())
}

/// Position of the `:` that separates key and value on a line (followed by a
/// space or end of line), skipping over a quoted key.
fn find_mapping_colon(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = 0;
    if text.starts_with('"') || text.starts_with('\'') {
        let q = bytes[0];
        i = 1;
        while i < bytes.len() {
            if bytes[i] == b'\\' && q == b'"' {
                i += 2;
                continue;
            }
            if bytes[i] == q {
                if q == b'\'' && i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                    i += 2;
                    continue;
                }
                break;
            }
            i += 1;
        }
        i += 1;
    }
    if text.starts_with("- ") || text == "-" || text.starts_with('#') {
        return None;
    }
    while i < bytes.len() {
        if bytes[i] == b'#' && i > 0 && (bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') {
            return None;
        }
        if bytes[i] == b':' && (i + 1 == bytes.len() || bytes[i + 1] == b' ' || bytes[i + 1] == b'\t') {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Strip a trailing ` # comment` from a plain/flow fragment (outside quotes).
fn strip_comment(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut in_s = false;
    let mut in_d = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if in_d {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_d = false;
            }
        } else if in_s {
            if c == b'\'' {
                in_s = false;
            }
        } else if c == b'"' {
            in_d = true;
        } else if c == b'\'' {
            in_s = true;
        } else if c == b'#' && (i == 0 || bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') {
            return &s[..i];
        }
        i += 1;
    }
    s
}

/// Parse the value starting on line `i` with inline text `rest` (after `key:`).
/// Returns the value and the index of the next unconsumed line.
fn parse_value_text(
    lines: &[Line],
    i: usize,
    rest: &str,
    parent_indent: usize,
    top_scalar: bool,
) -> Result<(Value, usize), String> {
    let no = lines[i].no;
    let t = strip_comment(rest).trim();
    if t.is_empty() {
        // Look ahead for a nested block.
        let mut j = i + 1;
        while j < lines.len() && {
            let tt = lines[j].text.trim_end();
            tt.is_empty() || tt.starts_with('#')
        } {
            j += 1;
        }
        if j < lines.len() {
            let l = &lines[j];
            let is_item = l.text == "-" || l.text.starts_with("- ");
            if is_item && l.indent >= parent_indent {
                return parse_block_seq(lines, j, l.indent);
            }
            if l.indent > parent_indent {
                return yerr(l.no, "nested mappings are not supported in front-matter");
            }
        }
        return Ok((Value::Null, i + 1));
    }
    let first = t.chars().next().unwrap_or(' ');
    match first {
        '[' => {
            // Flow sequence, possibly spanning lines.
            let mut text = String::from(strip_comment(rest).trim());
            let mut j = i + 1;
            loop {
                let chars: Vec<char> = text.chars().collect();
                match parse_flow_seq(&chars, 0) {
                    Ok((v, end)) => {
                        if chars[end..].iter().any(|c| !c.is_whitespace()) {
                            return yerr(no, "unexpected text after flow sequence");
                        }
                        return Ok((v, j));
                    }
                    Err(FlowErr::Incomplete) if j < lines.len() && lines[j].indent > parent_indent => {
                        text.push(' ');
                        text.push_str(strip_comment(lines[j].text).trim());
                        j += 1;
                    }
                    Err(FlowErr::Incomplete) => return yerr(no, "unterminated flow sequence"),
                    Err(FlowErr::Bad(m)) => return yerr(no, &m),
                }
            }
        }
        '{' => yerr(no, "flow mappings are not supported in front-matter"),
        '&' | '*' | '!' => yerr(no, "anchors, aliases and tags are not supported in front-matter"),
        '@' | '`' => yerr(no, "found character that cannot start any token"),
        '|' | '>' => parse_block_scalar(lines, i, t, parent_indent, top_scalar),
        '"' | '\'' => {
            // Quoted scalar, possibly multi-line.
            let mut text = String::from(rest.trim_start());
            let mut j = i + 1;
            loop {
                let chars: Vec<char> = text.chars().collect();
                match parse_quoted(&chars, 0) {
                    Ok((s, end)) => {
                        let tail: String = chars[end..].iter().collect();
                        if !strip_comment(&tail).trim().is_empty() {
                            return yerr(no, "unexpected text after quoted scalar");
                        }
                        return Ok((Value::Str(s), j));
                    }
                    Err(FlowErr::Incomplete) if j < lines.len() => {
                        text.push('\n');
                        text.push_str(lines[j].text);
                        j += 1;
                    }
                    Err(FlowErr::Incomplete) => return yerr(no, "unterminated quoted scalar"),
                    Err(FlowErr::Bad(m)) => return yerr(no, &m),
                }
            }
        }
        _ => {
            if (first == '-' || first == '?' || first == ':') && (t.len() == 1 || t[1..].starts_with(' ')) {
                return yerr(no, "block collections must start on their own line");
            }
            if find_mapping_colon(t).is_some() || t.ends_with(':') {
                return yerr(no, "mapping values are not allowed here");
            }
            // Plain scalar with possible continuation lines.
            let mut parts = vec![t.to_string()];
            let mut j = i + 1;
            let mut pending_blank = 0;
            while j < lines.len() {
                let l = &lines[j];
                let lt = l.text.trim_end();
                if lt.is_empty() {
                    pending_blank += 1;
                    j += 1;
                    continue;
                }
                if l.indent <= parent_indent && !top_scalar {
                    break;
                }
                if lt.starts_with('#') {
                    break;
                }
                let piece = strip_comment(lt).trim();
                if find_mapping_colon(piece).is_some() {
                    return yerr(l.no, "mapping values are not allowed here");
                }
                for _ in 0..pending_blank {
                    parts.push("\n".into());
                }
                pending_blank = 0;
                parts.push(piece.to_string());
                if strip_comment(lt).len() != lt.len() {
                    j += 1;
                    break;
                }
                j += 1;
            }
            // Fold: single newlines become spaces; blank lines become '\n'.
            let mut s = String::new();
            let mut prev_nl = false;
            for (k, p) in parts.iter().enumerate() {
                if p == "\n" {
                    s.push('\n');
                    prev_nl = true;
                    continue;
                }
                if k > 0 && !prev_nl {
                    s.push(' ');
                }
                s.push_str(p);
                prev_nl = false;
            }
            let v = if parts.len() == 1 { resolve_plain(&s) } else { Value::Str(s) };
            Ok((v, j))
        }
    }
}

fn parse_block_seq(lines: &[Line], start: usize, indent: usize) -> Result<(Value, usize), String> {
    let mut items = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let l = &lines[i];
        let tt = l.text.trim_end();
        if tt.is_empty() || tt.starts_with('#') {
            i += 1;
            continue;
        }
        if l.indent != indent || !(l.text == "-" || l.text.starts_with("- ")) {
            if l.indent > indent {
                return yerr(l.no, "unexpected indentation in sequence");
            }
            break;
        }
        let rest = if l.text == "-" { "" } else { &l.text[2..] };
        let inner = strip_comment(rest).trim();
        if inner.starts_with("- ") || inner == "-" || find_mapping_colon(inner).is_some() {
            return yerr(l.no, "nested collections are not supported in front-matter");
        }
        if inner.is_empty() {
            items.push(Value::Null);
            i += 1;
            continue;
        }
        let (v, next) = parse_value_text(lines, i, rest, indent, false)?;
        items.push(v);
        i = next;
    }
    Ok((Value::List(items), i))
}

fn parse_block_scalar(
    lines: &[Line],
    i: usize,
    header: &str,
    parent_indent: usize,
    _top: bool,
) -> Result<(Value, usize), String> {
    let no = lines[i].no;
    let folded = header.starts_with('>');
    let mut chomp = 'c';
    let mut explicit_indent: Option<usize> = None;
    for c in header[1..].chars() {
        match c {
            '-' => chomp = 's',
            '+' => chomp = 'k',
            '1'..='9' => explicit_indent = Some(c as usize - '0' as usize),
            ' ' => {}
            _ => return yerr(no, "invalid block scalar header"),
        }
    }
    let mut j = i + 1;
    let mut body: Vec<String> = Vec::new();
    let mut indent = explicit_indent.map(|e| parent_indent + e);
    while j < lines.len() {
        let l = &lines[j];
        if l.text.trim_end().is_empty() {
            body.push(String::new());
            j += 1;
            continue;
        }
        let ind = *indent.get_or_insert(l.indent);
        if l.indent < ind || l.indent <= parent_indent {
            break;
        }
        // Keep extra indentation beyond the block indent.
        body.push(format!("{}{}", " ".repeat(l.indent - ind), l.text));
        j += 1;
    }
    // trailing blank lines
    let mut trailing = 0;
    while body.last().map_or(false, |l| l.is_empty()) {
        body.pop();
        trailing += 1;
    }
    let mut s = String::new();
    if folded {
        let mut prev_more_indented = false;
        for (k, line) in body.iter().enumerate() {
            let more = line.starts_with(' ');
            if k > 0 {
                let prev = &body[k - 1];
                if prev.is_empty() || line.is_empty() || more || prev_more_indented {
                    s.push('\n');
                } else {
                    s.push(' ');
                }
            }
            prev_more_indented = more;
            s.push_str(line);
        }
    } else {
        s = body.join("\n");
    }
    match chomp {
        's' => {}
        'k' => {
            if !body.is_empty() {
                s.push('\n');
            }
            for _ in 0..trailing {
                s.push('\n');
            }
        }
        _ => {
            if !body.is_empty() {
                s.push('\n');
            }
        }
    }
    Ok((Value::Str(s), j))
}

enum FlowErr {
    Incomplete,
    Bad(String),
}

fn parse_quoted(chars: &[char], start: usize) -> Result<(String, usize), FlowErr> {
    let q = chars[start];
    let mut out = String::new();
    let mut i = start + 1;
    let mut raw_lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    while i < chars.len() {
        let c = chars[i];
        if q == '\'' {
            if c == '\'' {
                if i + 1 < chars.len() && chars[i + 1] == '\'' {
                    cur.push('\'');
                    i += 2;
                    continue;
                }
                raw_lines.push(cur);
                fold_quoted_lines(&raw_lines, &mut out);
                return Ok((out, i + 1));
            }
            if c == '\n' {
                raw_lines.push(std::mem::take(&mut cur));
                i += 1;
                continue;
            }
            cur.push(c);
            i += 1;
        } else {
            match c {
                '"' => {
                    raw_lines.push(cur);
                    fold_quoted_lines(&raw_lines, &mut out);
                    return Ok((out, i + 1));
                }
                '\\' => {
                    let e = match chars.get(i + 1) {
                        Some(e) => *e,
                        None => return Err(FlowErr::Incomplete),
                    };
                    i += 2;
                    let simple = match e {
                        '0' => Some('\0'),
                        'a' => Some('\u{7}'),
                        'b' => Some('\u{8}'),
                        't' | '\t' => Some('\t'),
                        'n' => Some('\n'),
                        'v' => Some('\u{b}'),
                        'f' => Some('\u{c}'),
                        'r' => Some('\r'),
                        'e' => Some('\u{1b}'),
                        ' ' => Some(' '),
                        '"' => Some('"'),
                        '/' => Some('/'),
                        '\\' => Some('\\'),
                        'N' => Some('\u{85}'),
                        '_' => Some('\u{a0}'),
                        'L' => Some('\u{2028}'),
                        'P' => Some('\u{2029}'),
                        _ => None,
                    };
                    if let Some(ch) = simple {
                        // escaped chars must survive folding: mark with a private sentinel
                        cur.push_str(&format!("\u{E000}{}", ch as u32));
                        cur.push('\u{E001}');
                        continue;
                    }
                    let n = match e {
                        'x' => 2,
                        'u' => 4,
                        'U' => 8,
                        '\n' => {
                            // escaped line break: join without space
                            cur.push('\u{E002}');
                            continue;
                        }
                        _ => return Err(FlowErr::Bad(format!("unknown escape character {:?}", e))),
                    };
                    if i + n > chars.len() {
                        return Err(FlowErr::Bad("truncated escape".into()));
                    }
                    let hex: String = chars[i..i + n].iter().collect();
                    let cp = u32::from_str_radix(&hex, 16).map_err(|_| FlowErr::Bad("bad escape".into()))?;
                    let ch = char::from_u32(cp).ok_or_else(|| FlowErr::Bad("bad escape".into()))?;
                    cur.push_str(&format!("\u{E000}{}", ch as u32));
                    cur.push('\u{E001}');
                    i += n;
                }
                '\n' => {
                    raw_lines.push(std::mem::take(&mut cur));
                    i += 1;
                }
                c => {
                    cur.push(c);
                    i += 1;
                }
            }
        }
    }
    Err(FlowErr::Incomplete)
}

/// YAML flow folding for quoted scalars, then expansion of escape sentinels.
fn fold_quoted_lines(lines: &[String], out: &mut String) {
    let mut s = String::new();
    let n = lines.len();
    let mut k = 0;
    while k < n {
        let mut line = lines[k].as_str();
        if k > 0 {
            line = line.trim_start_matches([' ', '\t']);
        }
        if k + 1 < n {
            line = line.trim_end_matches([' ', '\t']);
        }
        s.push_str(line);
        if k + 1 < n {
            if s.ends_with('\u{E002}') {
                s.pop();
                k += 1;
                // next line's leading whitespace is preserved after an escaped newline
                continue;
            }
            // count following empty lines
            let mut blanks = 0;
            while k + 1 + blanks < n - 1 && lines[k + 1 + blanks].trim().is_empty() {
                blanks += 1;
            }
            if blanks > 0 {
                for _ in 0..blanks {
                    s.push('\n');
                }
                k += blanks;
            } else {
                s.push(' ');
            }
        }
        k += 1;
    }
    // expand sentinels
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\u{E000}' {
            let mut j = i + 1;
            let mut num = String::new();
            while j < chars.len() && chars[j] != '\u{E001}' {
                num.push(chars[j]);
                j += 1;
            }
            if let Some(c) = num.parse::<u32>().ok().and_then(char::from_u32) {
                out.push(c);
            }
            i = j + 1;
            continue;
        }
        if chars[i] != '\u{E002}' {
            out.push(chars[i]);
        }
        i += 1;
    }
}

fn parse_flow_seq(chars: &[char], start: usize) -> Result<(Value, usize), FlowErr> {
    let mut i = start + 1;
    let mut items = Vec::new();
    let skip_ws = |i: &mut usize| {
        while *i < chars.len() && chars[*i].is_whitespace() {
            *i += 1;
        }
    };
    loop {
        skip_ws(&mut i);
        if i >= chars.len() {
            return Err(FlowErr::Incomplete);
        }
        match chars[i] {
            ']' => return Ok((Value::List(items), i + 1)),
            '"' | '\'' => {
                let (s, end) = parse_quoted(chars, i)?;
                items.push(Value::Str(s));
                i = end;
            }
            '[' | '{' => return Err(FlowErr::Bad("nested flow collections are not supported".into())),
            '&' | '*' | '!' | '@' | '`' | '|' | '>' => {
                return Err(FlowErr::Bad("unsupported character in flow sequence".into()))
            }
            ',' => return Err(FlowErr::Bad("expected a node, found ','".into())),
            _ => {
                let s0 = i;
                while i < chars.len() && chars[i] != ',' && chars[i] != ']' {
                    if chars[i] == ':' && (i + 1 >= chars.len() || chars[i + 1] == ' ' || chars[i + 1] == ',') {
                        return Err(FlowErr::Bad("flow mappings are not supported".into()));
                    }
                    if chars[i] == '[' || chars[i] == '{' || chars[i] == '}' {
                        return Err(FlowErr::Bad("unexpected character in flow sequence".into()));
                    }
                    i += 1;
                }
                let raw: String = chars[s0..i].iter().collect();
                items.push(resolve_plain(raw.trim()));
            }
        }
        skip_ws(&mut i);
        if i >= chars.len() {
            return Err(FlowErr::Incomplete);
        }
        match chars[i] {
            ',' => i += 1,
            ']' => return Ok((Value::List(items), i + 1)),
            _ => return Err(FlowErr::Bad("expected ',' or ']'".into())),
        }
    }
}

/// YAML 1.1 implicit resolution (PyYAML SafeLoader without timestamps).
fn resolve_plain(s: &str) -> Value {
    match s {
        "" | "~" | "null" | "Null" | "NULL" => return Value::Null,
        "yes" | "Yes" | "YES" | "true" | "True" | "TRUE" | "on" | "On" | "ON" => return Value::Bool(true),
        "no" | "No" | "NO" | "false" | "False" | "FALSE" | "off" | "Off" | "OFF" => return Value::Bool(false),
        _ => {}
    }
    if let Some(i) = resolve_int(s) {
        return Value::Int(i);
    }
    if let Some(f) = resolve_float(s) {
        return Value::Float(f);
    }
    Value::Str(s.to_string())
}

fn split_sign(s: &str) -> (i128, &str) {
    if let Some(r) = s.strip_prefix('-') {
        (-1, r)
    } else if let Some(r) = s.strip_prefix('+') {
        (1, r)
    } else {
        (1, s)
    }
}

fn resolve_int(s: &str) -> Option<i128> {
    let (sign, body) = split_sign(s);
    if body.is_empty() {
        return None;
    }
    let digits_ok = |t: &str, f: fn(char) -> bool| !t.is_empty() && t.chars().all(|c| c == '_' || f(c));
    let clean = |t: &str| t.replace('_', "");
    if let Some(b) = body.strip_prefix("0b") {
        if digits_ok(b, |c| c == '0' || c == '1') {
            return i128::from_str_radix(&clean(b), 2).ok().map(|v| sign * v);
        }
        return None;
    }
    if let Some(h) = body.strip_prefix("0x") {
        if digits_ok(h, |c| c.is_ascii_hexdigit()) {
            return i128::from_str_radix(&clean(h), 16).ok().map(|v| sign * v);
        }
        return None;
    }
    if body.len() > 1 && body.starts_with('0') {
        if digits_ok(&body[1..], |c| ('0'..='7').contains(&c)) {
            let c = clean(&body[1..]);
            if c.is_empty() {
                return Some(0);
            }
            return i128::from_str_radix(&c, 8).ok().map(|v| sign * v);
        }
        return None;
    }
    if body == "0" {
        return Some(0);
    }
    if body.contains(':') {
        // sexagesimal: [1-9][0-9_]*(:[0-5]?[0-9])+
        let mut parts = body.split(':');
        let head = parts.next()?;
        if !head.starts_with(|c: char| ('1'..='9').contains(&c)) || !digits_ok(head, |c| c.is_ascii_digit()) {
            return None;
        }
        let mut v: i128 = clean(head).parse().ok()?;
        for p in parts {
            if p.is_empty() || p.len() > 2 || !p.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            if p.len() == 2 && !('0'..='5').contains(&p.chars().next()?) {
                return None;
            }
            v = v.checked_mul(60)?.checked_add(p.parse::<i128>().ok()?)?;
        }
        return Some(sign * v);
    }
    if body.starts_with(|c: char| ('1'..='9').contains(&c)) && digits_ok(body, |c| c.is_ascii_digit()) {
        return clean(body).parse::<i128>().ok().map(|v| sign * v);
    }
    None
}

fn resolve_float(s: &str) -> Option<f64> {
    let (sign, body) = split_sign(s);
    let sign = sign as f64;
    match body {
        ".inf" | ".Inf" | ".INF" => return Some(sign * f64::INFINITY),
        ".nan" | ".NaN" | ".NAN" if s == body => return Some(f64::NAN),
        _ => {}
    }
    let (mant, exp) = match body.find(['e', 'E']) {
        Some(i) => (&body[..i], Some(&body[i + 1..])),
        None => (body, None),
    };
    if let Some(e) = exp {
        // exponent requires an explicit sign in YAML 1.1
        let ok = (e.starts_with('+') || e.starts_with('-')) && e.len() > 1 && e[1..].chars().all(|c| c.is_ascii_digit());
        if !ok {
            return None;
        }
    }
    let dot = mant.find('.')?;
    let (ip, fp) = (&mant[..dot], &mant[dot + 1..]);
    let ok_digits = |t: &str| t.chars().all(|c| c.is_ascii_digit() || c == '_');
    if ip.is_empty() {
        if s != body || fp.is_empty() || !fp.starts_with(|c: char| c.is_ascii_digit() || c == '_') || !ok_digits(fp) {
            return None;
        }
    } else if ip.contains(':') {
        return None; // sexagesimal floats: not worth supporting
    } else if !ip.starts_with(|c: char| c.is_ascii_digit()) || !ok_digits(ip) || !ok_digits(fp) {
        return None;
    }
    let text = format!(
        "{}{}.{}{}",
        if sign < 0.0 { "-" } else { "" },
        if ip.is_empty() { "0".to_string() } else { ip.replace('_', "") },
        if fp.is_empty() { "0".to_string() } else { fp.replace('_', "") },
        exp.map(|e| format!("e{}", e)).unwrap_or_default()
    );
    text.parse().ok()
}

// ---------------------------------------------------------------------------
// Schema validation (hand-written equivalent of spec/content-schema.json)
// ---------------------------------------------------------------------------

const PROPERTIES: &[&str] = &["title", "date", "updated", "summary", "tags", "draft", "aliases", "syndication"];
const DATE_PATTERN: &str = r"^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}(:\d{2})?(Z|[+-]\d{2}:?\d{2})?)?$";
const TAG_PATTERN: &str = "^[a-z0-9][a-z0-9-]*$";
const ALIAS_PATTERN: &str = "^/.+";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PathElem {
    Key(String),
    Idx(usize),
}

fn path_repr(p: &[PathElem]) -> String {
    let parts: Vec<String> = p
        .iter()
        .map(|e| match e {
            PathElem::Key(k) => py_repr_str(k),
            PathElem::Idx(i) => i.to_string(),
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

/// Python `re.search` treats `$` as matching before a single trailing newline.
fn strip_dollar_nl(s: &str) -> &str {
    s.strip_suffix('\n').unwrap_or(s)
}

fn digits(s: &[u8], n: usize) -> bool {
    s.len() >= n && s[..n].iter().all(|b| b.is_ascii_digit())
}

pub fn is_date_like(s: &str) -> bool {
    let b = strip_dollar_nl(s).as_bytes();
    if !(digits(b, 4) && b.get(4) == Some(&b'-') && digits(&b[5..], 2) && b.get(7) == Some(&b'-') && digits(&b[8..], 2)) {
        return false;
    }
    let mut r = &b[10..];
    if r.is_empty() {
        return true;
    }
    // T\d{2}:\d{2}
    if !(r[0] == b'T' && digits(&r[1..], 2) && r.get(3) == Some(&b':') && digits(&r[4..], 2)) {
        return false;
    }
    r = &r[6..];
    if r.len() >= 3 && r[0] == b':' && digits(&r[1..], 2) {
        r = &r[3..];
    }
    if r.is_empty() || r == b"Z" {
        return true;
    }
    if r[0] == b'+' || r[0] == b'-' {
        let t = &r[1..];
        return (t.len() == 4 && digits(t, 4)) || (t.len() == 5 && digits(t, 2) && t[2] == b':' && digits(&t[3..], 2));
    }
    false
}

pub fn is_tag(s: &str) -> bool {
    let s = strip_dollar_nl(s);
    let mut cs = s.chars();
    match cs.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    cs.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn is_alias(s: &str) -> bool {
    // ^/.+  (`.` excludes newline)
    let mut cs = s.chars();
    cs.next() == Some('/') && cs.next().map_or(false, |c| c != '\n')
}

/// RFC 3986 URI (absolute: scheme required), as `format: uri` intends.
pub fn is_uri(s: &str) -> bool {
    let colon = match s.find(':') {
        Some(c) => c,
        None => return false,
    };
    let scheme = &s[..colon];
    if scheme.is_empty()
        || !scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        || !scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        return false;
    }
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' {
            if i + 2 >= bytes.len() || !bytes[i + 1].is_ascii_hexdigit() || !bytes[i + 2].is_ascii_hexdigit() {
                return false;
            }
            i += 3;
            continue;
        }
        let ok = b.is_ascii_alphanumeric() || b"-._~:/?#[]@!$&'()*+,;=".contains(&b);
        if !ok {
            return false;
        }
        i += 1;
    }
    true
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Str(_) => "string",
        Value::List(_) => "array",
        Value::Bool(_) => "boolean",
        _ => "other",
    }
}

fn check_string(
    v: &Value,
    path: Vec<PathElem>,
    min_len: bool,
    pattern: Option<(&str, fn(&str) -> bool)>,
    uri: bool,
    errs: &mut Vec<(Vec<PathElem>, String)>,
) {
    if type_name(v) != "string" {
        errs.push((path.clone(), format!("{} is not of type 'string'", v.repr())));
    }
    if let Value::Str(s) = v {
        if min_len && s.chars().count() < 1 {
            errs.push((path.clone(), format!("{} should be non-empty", v.repr())));
        }
        if let Some((pat, f)) = pattern {
            if !f(s) {
                errs.push((path.clone(), format!("{} does not match {}", v.repr(), py_repr_str(pat))));
            }
        }
        if uri && !is_uri(s) {
            errs.push((path, format!("{} is not a 'uri'", v.repr())));
        }
    }
}

fn check_array(
    v: &Value,
    key: &str,
    pattern: Option<(&str, fn(&str) -> bool)>,
    uri: bool,
    errs: &mut Vec<(Vec<PathElem>, String)>,
) {
    let path = vec![PathElem::Key(key.to_string())];
    if type_name(v) != "array" {
        errs.push((path.clone(), format!("{} is not of type 'array'", v.repr())));
    }
    if let Value::List(items) = v {
        for (i, item) in items.iter().enumerate() {
            let mut p = path.clone();
            p.push(PathElem::Idx(i));
            check_string(item, p, false, pattern, uri, errs);
        }
        let mut seen: Vec<&Value> = Vec::new();
        let mut dup = false;
        for item in items {
            if seen.iter().any(|s| values_equal(s, item)) {
                dup = true;
                break;
            }
            seen.push(item);
        }
        if dup {
            errs.push((path, format!("{} has non-unique elements", v.repr())));
        }
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Float(y)) | (Value::Float(y), Value::Int(x)) => (*x as f64) == *y,
        _ => a == b,
    }
}

/// Validate front-matter; returns the formatted error list on failure.
pub fn validate(fm: &FrontMatter) -> Result<(), String> {
    let mut errs: Vec<(Vec<PathElem>, String)> = Vec::new();
    // additionalProperties
    let mut extras: Vec<&String> = fm.iter().map(|(k, _)| k).filter(|k| !PROPERTIES.contains(&k.as_str())).collect();
    extras.sort();
    extras.dedup();
    if !extras.is_empty() {
        let verb = if extras.len() == 1 { "was" } else { "were" };
        let list = extras.iter().map(|e| py_repr_str(e)).collect::<Vec<_>>().join(", ");
        errs.push((vec![], format!("Additional properties are not allowed ({} {} unexpected)", list, verb)));
    }
    // required
    if get(fm, "title").is_none() {
        errs.push((vec![], "'title' is a required property".into()));
    }
    // properties (schema order)
    for &prop in PROPERTIES {
        let v = match get(fm, prop) {
            Some(v) => v,
            None => continue,
        };
        let path = vec![PathElem::Key(prop.to_string())];
        match prop {
            "title" => check_string(v, path, true, None, false, &mut errs),
            "date" | "updated" => check_string(v, path, false, Some((DATE_PATTERN, is_date_like)), false, &mut errs),
            "summary" => check_string(v, path, false, None, false, &mut errs),
            "tags" => check_array(v, "tags", Some((TAG_PATTERN, is_tag)), false, &mut errs),
            "draft" => {
                if type_name(v) != "boolean" {
                    errs.push((path, format!("{} is not of type 'boolean'", v.repr())));
                }
            }
            "aliases" => check_array(v, "aliases", Some((ALIAS_PATTERN, is_alias)), false, &mut errs),
            "syndication" => check_array(v, "syndication", None, true, &mut errs),
            _ => {}
        }
    }
    if errs.is_empty() {
        return Ok(());
    }
    errs.sort_by(|a, b| a.0.cmp(&b.0));
    Err(errs.iter().map(|(p, m)| format!("{}: {}", path_repr(p), m)).collect::<Vec<_>>().join("; "))
}

/// Parse and validate the front-matter of `text`. Returns `(front-matter, body)`.
pub fn load(text: &str) -> Result<(FrontMatter, &str), LoadError> {
    let (yaml, body) = match split(text) {
        Some(x) => x,
        None => return Ok((Vec::new(), text)),
    };
    let fm = match parse_yaml(yaml).map_err(LoadError::Yaml)? {
        None => Vec::new(),
        Some(Value::Map(m)) => m,
        Some(Value::Null) => Vec::new(),
        Some(_) => return Err(LoadError::NotMapping),
    };
    Ok((fm, body))
}

pub enum LoadError {
    Yaml(String),
    NotMapping,
}

#[allow(dead_code)]
pub fn strip(s: &str) -> &str {
    py_strip(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_basics() {
        let y = "title: \"Hi: there\"\ndate: 2020-01-02\ntags: [a, 'b', c-d]\naliases:\n  - /x/\n  - /y/\ndraft: yes\nsummary: plain text # comment\n";
        let v = parse_yaml(y).unwrap().unwrap();
        let m = match v {
            Value::Map(m) => m,
            _ => panic!(),
        };
        assert_eq!(get(&m, "title"), Some(&Value::Str("Hi: there".into())));
        assert_eq!(get(&m, "date"), Some(&Value::Str("2020-01-02".into())));
        assert_eq!(
            get(&m, "tags"),
            Some(&Value::List(vec![Value::Str("a".into()), Value::Str("b".into()), Value::Str("c-d".into())]))
        );
        assert_eq!(get(&m, "aliases").unwrap().repr(), "['/x/', '/y/']");
        assert_eq!(get(&m, "draft"), Some(&Value::Bool(true)));
        assert_eq!(get(&m, "summary"), Some(&Value::Str("plain text".into())));
        assert!(validate(&m).is_ok());
    }

    #[test]
    fn schema_errors() {
        let m = match parse_yaml("title: 12\ntags: [Bad, Bad]\nfoo: 1\n").unwrap().unwrap() {
            Value::Map(m) => m,
            _ => panic!(),
        };
        let e = validate(&m).unwrap_err();
        assert_eq!(
            e,
            "[]: Additional properties are not allowed ('foo' was unexpected); ['tags']: ['Bad', 'Bad'] has non-unique elements; ['tags', 0]: 'Bad' does not match '^[a-z0-9][a-z0-9-]*$'; ['tags', 1]: 'Bad' does not match '^[a-z0-9][a-z0-9-]*$'; ['title']: 12 is not of type 'string'"
        );
    }

    #[test]
    fn split_front_matter() {
        assert_eq!(split("---\na: 1\n---\nbody"), Some(("a: 1", "body")));
        assert_eq!(split("---\n---\nx\n---\n"), Some(("---\nx", "")));
        assert_eq!(split("no"), None);
    }
}
