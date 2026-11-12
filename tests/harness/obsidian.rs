//! The vault's committed Obsidian settings stamp `updated_at` in the form
//! the build reads (spec/content-schema.md,
//! ADR 2026-11-12-obsidian-stamps-updated-at).

use posse_bliki_build::date::Stamp;
use posse_bliki_build::json::{self, Json};

use crate::support::{content_dir, read_as_built};

fn settings(file: &str) -> Json {
    let path = content_dir().join(".obsidian").join(file);
    json::parse(&read_as_built(&path)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

#[test]
fn linter_is_turned_on_in_the_vault() {
    let on = settings("community-plugins.json");
    assert!(
        on.as_arr().iter().any(|p| p.as_str() == Some("obsidian-linter")),
        "community-plugins.json doesn't list obsidian-linter"
    );
}

#[test]
fn linter_stamps_updated_at_in_utc() {
    let data = settings("plugins/obsidian-linter/data.json");
    let rule = data
        .get("ruleConfigs")
        .and_then(|r| r.get("yaml-timestamp"))
        .expect("no yaml-timestamp rule");
    let is = |key: &str, want: &str| rule.get(key) == Some(&Json::Lit(want.to_string()));
    assert!(is("enabled", "true"), "the YAML Timestamp rule is off");
    assert!(is("date-modified", "true"), "the modified date is off");
    assert_eq!(rule.str_at("date-modified-key"), Some("updated_at"));
    assert!(is("convert-to-utc", "true"), "stamps would be in local time");
    assert!(
        is("date-created", "false"),
        "a created date would be a field the build doesn't know"
    );
    // What the format writes for one moment is something the build reads.
    let format = rule.str_at("format").expect("no format");
    let written = [
        ("YYYY", "2026"),
        ("MM", "11"),
        ("DD", "12"),
        ("HH", "04"),
        ("mm", "27"),
        ("ss", "28"),
        ("[Z]", "Z"),
    ]
    .iter()
    .fold(format.to_string(), |s, (token, value)| s.replace(token, value));
    assert_eq!(
        Stamp::parse(&written).map(|s| s.iso()).as_deref(),
        Some("2026-11-12T04:27:28Z"),
        "format {format:?} writes {written:?}"
    );
}
