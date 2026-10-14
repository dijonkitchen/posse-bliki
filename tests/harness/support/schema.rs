//! A JSON Schema (draft 2020-12) validator for the keywords
//! spec/content-schema.json uses. It replaces the jsonschema package
//! (ADR 0010) and reads the schema file itself, so the file stays the single
//! statement of the front-matter contract and is checked independently of the
//! build's own `frontmatter::validate`. A keyword it does not know panics, so
//! a schema change can't be silently ignored. Like jsonschema's default
//! validator, it treats `format` as an annotation and does not enforce it.

use posse_bliki_build::frontmatter::Value;
use posse_bliki_build::json::Json;

use super::regex::Regex;

const ANNOTATIONS: &[&str] = &["$schema", "$id", "title", "description", "format"];

/// Every way `v` breaks `schema`, as `"path: message"`.
pub fn validate(schema: &Json, v: &Value) -> Vec<String> {
    let mut errs = Vec::new();
    check(schema, v, "$", &mut errs);
    errs
}

fn type_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Int(_) => "integer",
        Value::Float(_) => "number",
        Value::Str(_) => "string",
        Value::List(_) => "array",
        Value::Map(_) => "object",
    }
}

fn check(schema: &Json, v: &Value, path: &str, errs: &mut Vec<String>) {
    let Json::Obj(keywords) = schema else {
        panic!("schema at {path} is not an object");
    };
    for (key, arg) in keywords {
        match key.as_str() {
            k if ANNOTATIONS.contains(&k) => {}
            "type" => {
                let want = arg.as_str().expect("type is a string");
                let got = type_of(v);
                if got != want && !(want == "number" && got == "integer") {
                    errs.push(format!("{path}: expected {want}, got {got}"));
                }
            }
            "required" => {
                if let Value::Map(m) = v {
                    for r in arg.as_arr() {
                        let r = r.as_str().unwrap();
                        if !m.iter().any(|(k, _)| k == r) {
                            errs.push(format!("{path}: missing required property {r:?}"));
                        }
                    }
                }
            }
            "properties" => {
                if let Value::Map(m) = v {
                    for (k, val) in m {
                        if let Some(sub) = arg.get(k) {
                            check(sub, val, &format!("{path}.{k}"), errs);
                        }
                    }
                }
            }
            "additionalProperties" => {
                assert!(
                    matches!(arg, Json::Lit(l) if l == "false"),
                    "only additionalProperties: false is supported"
                );
                let known = schema.get("properties");
                if let Value::Map(m) = v {
                    for (k, _) in m {
                        if known.and_then(|p| p.get(k)).is_none() {
                            errs.push(format!("{path}: unexpected property {k:?}"));
                        }
                    }
                }
            }
            "minLength" => {
                let min: usize = lit(arg).parse().expect("minLength is a number");
                if let Value::Str(s) = v {
                    if s.chars().count() < min {
                        errs.push(format!("{path}: shorter than {min}"));
                    }
                }
            }
            "pattern" => {
                let pat = arg.as_str().expect("pattern is a string");
                if let Value::Str(s) = v {
                    if !Regex::new(pat).is_match(s) {
                        errs.push(format!("{path}: {s:?} does not match {pat:?}"));
                    }
                }
            }
            "items" => {
                if let Value::List(items) = v {
                    for (i, item) in items.iter().enumerate() {
                        check(arg, item, &format!("{path}[{i}]"), errs);
                    }
                }
            }
            "uniqueItems" => {
                if lit(arg) == "true" {
                    if let Value::List(items) = v {
                        let reprs: Vec<String> = items.iter().map(Value::repr).collect();
                        for (i, r) in reprs.iter().enumerate() {
                            if reprs[..i].contains(r) {
                                errs.push(format!("{path}: duplicate item {r}"));
                            }
                        }
                    }
                }
            }
            other => panic!("schema keyword {other:?} is not supported by tests/harness/support/schema.rs"),
        }
    }
}

fn lit(j: &Json) -> &str {
    match j {
        Json::Lit(l) => l,
        other => panic!("expected a literal, got {other:?}"),
    }
}
