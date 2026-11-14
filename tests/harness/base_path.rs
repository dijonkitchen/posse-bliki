//! Built for a GitHub Pages project URL, every internal link stays under the
//! sub-path (spec/output-contract.md, "Serving under a sub-path").

use std::path::PathBuf;
use std::sync::OnceLock;

use crate::support::{build, config_str, content_dir, files_with_ext, read, rel, strip_fragment, temp_dir};

const PREVIEW: &str = "https://dijonkitchen.github.io/posse-bliki";
const PREFIX: &str = "/posse-bliki/";

fn sub_site() -> &'static PathBuf {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let out = temp_dir("sub");
        build(&content_dir(), &out, &["--base-url", PREVIEW]).expect("build under a sub-path");
        out
    })
}

/// Every `href="..."` and meta-refresh `url=...` target in `html`.
fn urls(html: &str) -> Vec<String> {
    let mut out: Vec<String> = crate::support::hrefs(html)
        .into_iter()
        .map(|h| strip_fragment(h).to_string())
        .collect();
    for (i, _) in html.match_indices("url=") {
        let rest = &html[i + 4..];
        out.push(rest[..rest.find('"').unwrap_or(rest.len())].to_string());
    }
    out
}

#[test]
fn links_resolve_under_sub_path() {
    let site = sub_site();
    let mut failures = Vec::new();
    for f in files_with_ext(site, "html") {
        for url in urls(&read(&f)) {
            let url = match url.strip_prefix(PREVIEW) {
                Some(rest) => format!("{PREFIX}{}", rest.trim_start_matches('/')),
                None => url,
            };
            if !url.starts_with('/') || url.starts_with("//") {
                continue;
            }
            let Some(path) = url.strip_prefix(PREFIX) else {
                failures.push(format!("{} -> {url} (not under {PREFIX})", rel(&f, site)));
                continue;
            };
            if !(site.join(path).is_file() || site.join(path).join("index.html").is_file()) {
                failures.push(format!("{} -> {url}", rel(&f, site)));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "broken links under sub-path:\n{}",
        failures.join("\n")
    );
}

#[test]
fn feeds_use_base_url() {
    let site = sub_site();
    let base = format!("{}/", config_str("base_url").trim_end_matches('/'));
    for name in ["index.xml", "feed.json", "sitemap.xml"] {
        let text = read(&site.join(name));
        assert!(!text.contains(&base), "{name} still uses {base}");
        assert!(text.contains(&format!("{PREVIEW}/")), "{name} does not use {PREVIEW}");
    }
    assert!(!read(&site.join("feed.json")).contains("href=\\\"/notes/"));
    assert!(!read(&site.join("index.xml")).contains("href=&quot;/notes/"));
}

#[test]
fn inline_style_and_script_work_under_sub_path() {
    // The style picker's root and the house style's font URLs take the
    // sub-path, and the policy's hashes still match the blocks once the
    // page's links are prefixed (ADR 2026-11-11-inline-house-style-and-script).
    let site = sub_site();
    for f in files_with_ext(site, "html") {
        let html = read(&f);
        let page = rel(&f, site);
        assert!(
            html.contains(&format!("<script data-root=\"{PREFIX}\" data-styles=\"")),
            "{page}: the style picker's root is not {PREFIX}"
        );
        let style = crate::csp::inline_text(&html, "style");
        let urls: Vec<&str> = style
            .match_indices("url(\"")
            .map(|(i, _)| style[i + 5..].split('"').next().unwrap())
            .collect();
        assert!(!urls.is_empty(), "{page}: no font urls in the house style");
        for url in urls {
            let path = url
                .strip_prefix(PREFIX)
                .unwrap_or_else(|| panic!("{page}: url({url}) not under {PREFIX}"));
            assert!(site.join(path).is_file(), "{page}: url({url}) does not resolve");
        }
        assert!(
            html.contains(&crate::csp::expected_policy(&html)),
            "{page}: the policy's hashes don't match the inline blocks"
        );
    }
    assert!(
        read(&site.join("style.css")).contains(&format!("url(\"{PREFIX}fonts/")),
        "style.css font urls not under {PREFIX}"
    );
}
