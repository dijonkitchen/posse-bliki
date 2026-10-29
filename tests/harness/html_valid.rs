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
            let error_page = p.ends_with("404.html") || p.ends_with("offline.html");
            if error_page && needle.starts_with("<link rel=\"canonical\"") {
                continue; // 404 and offline pages don't get a canonical
            }
            assert!(text.contains(needle), "{}: missing {needle:?}", rel(p, site()));
        }
    }
}

#[test]
fn only_the_style_picker_script() {
    // The site must work with JS off; the one script only remembers the
    // picked style (ADR 2026-10-18-style-picker).
    let allowed = "<script src=\"/style-picker.js\"></script>";
    for (p, text) in html_files() {
        let lower = text.to_lowercase();
        let scripts = lower.matches("<script").count();
        let picker = usize::from(text.contains(allowed));
        assert_eq!(scripts, picker, "{}: unexpected <script>", rel(p, site()));
    }
}
