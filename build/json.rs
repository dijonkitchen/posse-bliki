//! JSON output byte-compatible with Python's `json.dumps(obj, indent=2)`
//! (default `ensure_ascii=True`).

pub enum Json {
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
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
}
