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

/// A note needs `date` to be a post, or `updated` to be an evergreen note
/// (ADR 2026-11-07-evergreen-notes).
#[test]
fn notes_have_date_or_updated() {
    let content = content_dir();
    for md in files_with_ext(&content.join("notes"), "md") {
        let fm = front_matter(&read(&md));
        if frontmatter::get(&fm, "draft").is_some_and(Value::truthy) {
            continue;
        }
        assert!(
            frontmatter::get(&fm, "date").is_some() || frontmatter::get(&fm, "updated").is_some(),
            "{} has neither `date` (a post) nor `updated` (an evergreen note)",
            rel(&md, &content)
        );
    }
}
