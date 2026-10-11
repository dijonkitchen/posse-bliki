//! JSON output byte-compatible with Python's `json.dumps(obj, indent=2)`
//! (default `ensure_ascii=True`), and a small parser for the JSON the build
//! reads back (a previous `feed.json`, webmention.io's jf2 export).

#[derive(Debug, PartialEq)]
pub enum Json {
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
    /// A number, `true`, `false` or `null`, kept as its source text.
    Lit(String),
}

impl Json {
    /// The value under `key` when this is an object.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(items) => items.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> &[Json] {
        match self {
            Json::Arr(items) => items,
            _ => &[],
        }
    }

    /// `get(key)` as a string, `None` when missing or not a string.
    pub fn str_at(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Json::as_str)
    }
}

pub fn s(v: &str) -> Json {
    Json::Str(v.to_string())
}

fn quote(v: &str, out: &mut String) {
    out.push('"');
    for c in v.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            ' '..='~' => out.push(c),
            c => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{:04x}", unit));
                }
            }
        }
    }
    out.push('"');
}

fn write(v: &Json, level: usize, out: &mut String) {
    match v {
        Json::Str(x) => quote(x, out),
        Json::Lit(x) => out.push_str(x),
        Json::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.push_str(&"  ".repeat(level + 1));
                write(item, level + 1, out);
            }
            out.push('\n');
            out.push_str(&"  ".repeat(level));
            out.push(']');
        }
        Json::Obj(items) => {
            if items.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, item)) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.push_str(&"  ".repeat(level + 1));
                quote(k, out);
                out.push_str(": ");
                write(item, level + 1, out);
            }
            out.push('\n');
            out.push_str(&"  ".repeat(level));
            out.push('}');
        }
    }
}

pub fn dumps(v: &Json) -> String {
    let mut out = String::new();
    write(v, 0, &mut out);
    out
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn err<T>(&self, what: &str) -> Result<T, String> {
        Err(format!("invalid JSON at byte {}: {}", self.i, what))
    }

    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.ws();
        if self.b.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > 128 {
            return self.err("nested too deeply");
        }
        self.ws();
        match self.b.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                let mut items = Vec::new();
                if self.eat(b'}') {
                    return Ok(Json::Obj(items));
                }
                loop {
                    self.ws();
                    if self.b.get(self.i) != Some(&b'"') {
                        return self.err("expected object key");
                    }
                    let k = self.string()?;
                    if !self.eat(b':') {
                        return self.err("expected ':'");
                    }
                    items.push((k, self.value(depth + 1)?));
                    if self.eat(b'}') {
                        return Ok(Json::Obj(items));
                    }
                    if !self.eat(b',') {
                        return self.err("expected ',' or '}'");
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut items = Vec::new();
                if self.eat(b']') {
                    return Ok(Json::Arr(items));
                }
                loop {
                    items.push(self.value(depth + 1)?);
                    if self.eat(b']') {
                        return Ok(Json::Arr(items));
                    }
                    if !self.eat(b',') {
                        return self.err("expected ',' or ']'");
                    }
                }
            }
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(_) => {
                let start = self.i;
                while self.i < self.b.len()
                    && matches!(self.b[self.i], b'0'..=b'9' | b'a'..=b'z' | b'-' | b'+' | b'.' | b'E')
                {
                    self.i += 1;
                }
                let lit = std::str::from_utf8(&self.b[start..self.i]).unwrap_or("");
                let ok = matches!(lit, "true" | "false" | "null")
                    || (!lit.is_empty() && lit.parse::<f64>().is_ok() && !lit.starts_with('+'));
                if ok {
                    Ok(Json::Lit(lit.to_string()))
                } else {
                    self.i = start;
                    self.err("unexpected value")
                }
            }
            None => self.err("unexpected end of input"),
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let h = self.b.get(self.i..self.i + 4).and_then(|h| std::str::from_utf8(h).ok());
        match h.and_then(|h| u32::from_str_radix(h, 16).ok()) {
            Some(v) => {
                self.i += 4;
                Ok(v)
            }
            None => self.err("bad \\u escape"),
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.i += 1; // opening quote
        let mut out = String::new();
        loop {
            let start = self.i;
            while self.i < self.b.len() && self.b[self.i] != b'"' && self.b[self.i] != b'\\' {
                if self.b[self.i] < 0x20 {
                    return self.err("control character in string");
                }
                self.i += 1;
            }
            match std::str::from_utf8(&self.b[start..self.i]) {
                Ok(s) => out.push_str(s),
                Err(_) => return self.err("invalid UTF-8"),
            }
            match self.b.get(self.i) {
                Some(b'"') => {
                    self.i += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.i += 1;
                    let c = self.b.get(self.i).copied();
                    self.i += 1;
                    match c {
                        Some(b'"') => out.push('"'),
                        Some(b'\\') => out.push('\\'),
                        Some(b'/') => out.push('/'),
                        Some(b'b') => out.push('\u{8}'),
                        Some(b'f') => out.push('\u{c}'),
                        Some(b'n') => out.push('\n'),
                        Some(b'r') => out.push('\r'),
                        Some(b't') => out.push('\t'),
                        Some(b'u') => {
                            let hi = self.hex4()?;
                            let cp = if (0xd800..0xdc00).contains(&hi) && self.b.get(self.i..self.i + 2) == Some(b"\\u") {
                                self.i += 2;
                                let lo = self.hex4()?;
                                if !(0xdc00..0xe000).contains(&lo) {
                                    return self.err("bad surrogate pair");
                                }
                                0x10000 + ((hi - 0xd800) << 10) + (lo - 0xdc00)
                            } else {
                                hi
                            };
                            out.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                        }
                        _ => return self.err("bad escape"),
                    }
                }
                _ => return self.err("unterminated string"),
            }
        }
    }
}

/// Parse a complete JSON document.
pub fn parse(text: &str) -> Result<Json, String> {
    let mut p = Parser { b: text.as_bytes(), i: 0 };
    let v = p.value(0)?;
    p.ws();
    if p.i != p.b.len() {
        return p.err("trailing data");
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_compatible() {
        let v = Json::Obj(vec![
            ("a".into(), s("é😀\u{7f}\u{2028}\"\\/")),
            ("b".into(), Json::Arr(vec![])),
            ("c".into(), Json::Obj(vec![])),
            ("d".into(), Json::Arr(vec![s("x")])),
        ]);
        assert_eq!(
            dumps(&v),
            "{\n  \"a\": \"\\u00e9\\ud83d\\ude00\\u007f\\u2028\\\"\\\\/\",\n  \"b\": [],\n  \"c\": {},\n  \"d\": [\n    \"x\"\n  ]\n}"
        );
    }

    #[test]
    fn parses_round_trip() {
        let src = "{\"a\": [1, -2.5e3, true, null], \"b\": \"x\\u00e9\\ud83d\\ude00\\n\", \"c\": {}}";
        let v = parse(src).unwrap();
        assert_eq!(v.get("a").unwrap().as_arr().len(), 4);
        assert_eq!(v.str_at("b"), Some("x\u{e9}\u{1f600}\n"));
        assert_eq!(dumps(&parse(&dumps(&v)).unwrap()), dumps(&v));
    }

    #[test]
    fn rejects_bad_input() {
        for bad in ["", "{", "[1,]", "{\"a\" 1}", "\"x", "tru", "1 2", "{\"a\":1,}"] {
            assert!(parse(bad).is_err(), "{:?} should not parse", bad);
        }
    }
}
