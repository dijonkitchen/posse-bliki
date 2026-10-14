//! Every emitted HTML page parses cleanly and carries the universal invariants
//! (spec/output-contract.md, "Universal").

use crate::support::{html, html_files, rel, site};

#[test]
fn at_least_a_few_pages_exist() {
    assert!(
        html_files().len() >= 4,
        "expected several HTML files, got {}",
        html_files().len()
    );
}

#[test]
fn strict_html5_parses() {
    let failures: Vec<String> = html_files()
        .iter()
        .filter_map(|(p, text)| html::parse(text).err().map(|e| format!("{}: {e}", rel(p, site()))))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn universal_invariants() {
    let needles = [
        "<!DOCTYPE html>",
        "<html lang=\"en\"",
        "<meta charset=\"utf-8\">",
        "<meta name=\"viewport\"",
        "<link rel=\"canonical\"",
        "<link rel=\"alternate\" type=\"application/rss+xml\"",
        "<link rel=\"alternate\" type=\"application/feed+json\"",
        "<link rel=\"webmention\"",
        "<link rel=\"pingback\"",
    ];
    for (p, text) in html_files() {
        for needle in needles {
            if p.ends_with("404.html") && needle.starts_with("<link rel=\"canonical\"") {
                continue; // 404s don't get a canonical
            }
            assert!(text.contains(needle), "{}: missing {needle:?}", rel(p, site()));
        }
    }
}

#[test]
fn no_script_tags() {
    // The site must work with JS off.
    for (p, text) in html_files() {
        assert!(
            !text.to_lowercase().contains("<script"),
            "{}: contains <script>",
            rel(p, site())
        );
    }
}
