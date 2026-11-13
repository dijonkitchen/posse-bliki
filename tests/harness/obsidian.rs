//! The vault's committed Obsidian settings stamp `updated_at` in the form
//! the build reads, and only in `content/` (spec/content-schema.md,
//! ADR 2026-11-12-obsidian-stamps-updated-at). The vault is the repo root,
//! so its settings are the root `.obsidian/`.

use posse_bliki_build::date::Stamp;
use posse_bliki_build::json::{self, Json};

use crate::support::{read_as_built, repo};

fn settings(file: &str) -> Json {
    let path = repo().join(".obsidian").join(file);
    json::parse(&read_as_built(&path)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

fn linter() -> Json {
    settings("plugins/obsidian-linter/data.json")
}

fn timestamp_rule(data: &Json) -> &Json {
    data.get("ruleConfigs")
        .and_then(|r| r.get("yaml-timestamp"))
        .expect("no yaml-timestamp rule")
}

fn is(rule: &Json, key: &str, want: &str) -> bool {
    rule.get(key) == Some(&Json::Lit(want.to_string()))
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
    let data = linter();
    let rule = timestamp_rule(&data);
    assert!(is(rule, "enabled", "true"), "the YAML Timestamp rule is off");
    assert!(is(rule, "date-modified", "true"), "the modified date is off");
    assert_eq!(rule.str_at("date-modified-key"), Some("updated_at"));
    assert!(is(rule, "convert-to-utc", "true"), "stamps would be in local time");
    assert!(
        is(rule, "date-created", "false"),
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

#[test]
fn linter_leaves_files_outside_content_alone() {
    // Linter tests each pattern as a JavaScript RegExp against the file's
    // path from the vault root, and skips the files that match.
    let data = linter();
    let ignored: Vec<&str> = data
        .get("filesToIgnore")
        .map(|f| f.as_arr().iter().filter_map(|p| p.str_at("match")).collect())
        .unwrap_or_default();
    assert!(
        ignored.contains(&"^(?!content/)"),
        "Linter would stamp README.md, the specs and the ADRs too: filesToIgnore is {ignored:?}"
    );
}
