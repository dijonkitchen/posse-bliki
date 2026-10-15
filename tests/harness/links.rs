//! Every internal href resolves to an emitted file (spec/output-contract.md).

use std::path::Path;

use crate::support::{hrefs, html_files, rel, site, strip_fragment};

/// The file a static host would serve for URL path `href`.
fn resolves(site: &Path, href: &str) -> bool {
    let path = href.trim_start_matches('/');
    if path.is_empty() {
        return site.join("index.html").is_file();
    }
    site.join(path).is_file()
        || site.join(path).join("index.html").is_file()
        || (!path.ends_with('/') && site.join(format!("{path}.html")).is_file())
}

#[test]
fn no_broken_internal_links() {
    let root = site();
    let mut failures = Vec::new();
    for (p, text) in html_files() {
        for href in hrefs(text).into_iter().map(strip_fragment) {
            if href.starts_with('/') && !href.starts_with("//") && !resolves(root, href) {
                failures.push(format!("{} -> {href}", rel(p, root)));
            }
        }
    }
    assert!(failures.is_empty(), "broken internal links:\n{}", failures.join("\n"));
}
