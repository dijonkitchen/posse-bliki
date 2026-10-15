//! Trailing slashes, no .html in URLs, slug = filename stem (spec/url-policy.md).

use crate::support::{hrefs, html_files, rel, site, strip_fragment};

#[test]
fn pages_emitted_at_trailing_slash_directories() {
    let root = site();
    let expected = [
        "index.html",
        "about/index.html",
        "colophon/index.html",
        "notes/index.html",
        "notes/how-this-bliki-works/index.html",
    ];
    let missing: Vec<&str> = expected.into_iter().filter(|p| !root.join(p).is_file()).collect();
    assert!(missing.is_empty(), "missing expected pages: {missing:?}");
}

#[test]
fn special_files_emitted() {
    for f in ["index.xml", "feed.json", "sitemap.xml", "robots.txt", "404.html"] {
        assert!(site().join(f).is_file(), "missing {f}");
    }
}

#[test]
fn no_html_extension_in_internal_links() {
    // Internal links use trailing slashes, never explicit .html (except /404.html).
    let mut failures = Vec::new();
    for (p, text) in html_files() {
        for href in hrefs(text).into_iter().map(strip_fragment) {
            if href.starts_with('/') && href.ends_with(".html") && href != "/404.html" {
                failures.push(format!("{} -> {href}", rel(p, site())));
            }
        }
    }
    assert!(failures.is_empty(), "internal .html links:\n{}", failures.join("\n"));
}
