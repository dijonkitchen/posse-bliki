//! Every `content/**/*.md` front-matter validates against
//! spec/content-schema.json (spec/content-schema.md).
//!
//! The YAML is read with the build's own parser (Rust has no YAML in std);
//! the schema check is independent of the build's `frontmatter::validate`.

use posse_bliki_build::frontmatter::{self, Value};
use posse_bliki_build::json;

use crate::support::{content_dir, files_with_ext, read, rel, repo, schema};

/// The front-matter mapping of `text`: the YAML between the first two `---` lines.
fn front_matter(text: &str) -> Vec<(String, Value)> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Vec::new();
    };
    let yaml = &rest[..rest.find("---\n").expect("unterminated front-matter")];
    match frontmatter::parse_yaml(yaml).expect("front-matter is YAML") {
        None => Vec::new(),
        Some(Value::Map(m)) => m,
        Some(other) => panic!("front-matter is not a mapping: {}", other.repr()),
    }
}

#[test]
fn frontmatter_validates() {
    let schema = json::parse(&read(&repo().join("spec/content-schema.json"))).expect("schema is JSON");
    let content = content_dir();
    let mut failures = Vec::new();
    for md in files_with_ext(&content, "md") {
        let fm = Value::Map(front_matter(&read(&md)));
        for e in schema::validate(&schema, &fm) {
            failures.push(format!("{}: {e}", rel(&md, &content)));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A note needs `created_at` to be a post, or `updated_at` to be an
/// evergreen note (ADR 2026-11-07-evergreen-notes).
#[test]
fn notes_have_created_at_or_updated_at() {
    let content = content_dir();
    for md in files_with_ext(&content.join("notes"), "md") {
        let fm = front_matter(&read(&md));
        if frontmatter::get(&fm, "draft").is_some_and(Value::truthy) {
            continue;
        }
        assert!(
            frontmatter::get(&fm, "created_at").is_some() || frontmatter::get(&fm, "updated_at").is_some(),
            "{} has neither `created_at` (a post) nor `updated_at` (an evergreen note)",
            rel(&md, &content)
        );
    }
}

/// `updated_at` is a day or a UTC datetime, and `updated` is no longer a
/// field (ADR 2026-11-12-updated-at-utc-datetimes). The schema file and the
/// build's own check agree on every sample.
#[test]
fn updated_at_is_a_day_or_a_utc_datetime() {
    let schema = json::parse(&read(&repo().join("spec/content-schema.json"))).expect("schema is JSON");
    let errors = |key: &str, value: &str| {
        let fm = Value::Map(vec![
            ("title".to_string(), Value::Str("t".to_string())),
            (key.to_string(), Value::Str(value.to_string())),
        ]);
        schema::validate(&schema, &fm)
    };
    for ok in [
        "2026-11-12",
        "2026-11-12T04:27:28Z",
        "2026-11-12T04:27Z",
        "2026-11-12T04:27:28+00:00",
    ] {
        assert!(errors("updated_at", ok).is_empty(), "schema refuses {ok}");
        assert!(frontmatter::is_updated_at_like(ok), "build refuses {ok}");
    }
    for bad in [
        "2026-11-11T20:27:28-08:00",
        "2026-11-11T20:27:28",
        "2026-11-12T04:27:28+0000",
        "2026-11-12 04:27:28Z",
        "November 12th 2026",
    ] {
        assert!(!errors("updated_at", bad).is_empty(), "schema accepts {bad}");
        assert!(!frontmatter::is_updated_at_like(bad), "build accepts {bad}");
    }
    assert!(
        !errors("updated", "2026-11-12").is_empty(),
        "schema still has `updated`"
    );
}

/// `created_at` is a day, and `date` is no longer a field
/// (ADR 2026-11-13-created-at). The schema file and the build's own check
/// agree on every sample.
#[test]
fn created_at_is_a_day() {
    let schema = json::parse(&read(&repo().join("spec/content-schema.json"))).expect("schema is JSON");
    let errors = |key: &str, value: &str| {
        let fm = Value::Map(vec![
            ("title".to_string(), Value::Str("t".to_string())),
            (key.to_string(), Value::Str(value.to_string())),
        ]);
        schema::validate(&schema, &fm)
    };
    assert!(errors("created_at", "2026-11-13").is_empty(), "schema refuses a day");
    assert!(frontmatter::is_day_like("2026-11-13"), "build refuses a day");
    for bad in [
        "2026-11-13T09:00:00Z",
        "2026-11-13T09:00",
        "13 November 2026",
        "2026-11-13 ",
    ] {
        assert!(!errors("created_at", bad).is_empty(), "schema accepts {bad:?}");
        assert!(!frontmatter::is_day_like(bad), "build accepts {bad:?}");
    }
    assert!(!errors("date", "2026-11-13").is_empty(), "schema still has `date`");
}
