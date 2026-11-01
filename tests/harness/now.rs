//! The now page, after Derek Sivers's (spec/content-schema.md, ADR 2026-11-01-now-page).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::support::{build, content_dir, files_with_ext, html_files, read, rel, site, temp_dir, vault_copy};

const PLACEHOLDER: &str = "TODO(JC)";

/// The vault with `now.md` published, as it will be once JC fills it in.
fn published_now() -> &'static PathBuf {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let vault = vault_copy("now");
        let now = vault.join("now.md");
        let kept: Vec<String> = read(&now)
            .lines()
            .filter(|l| !l.starts_with("draft:"))
            .map(String::from)
            .collect();
        fs::write(&now, kept.join("\n")).unwrap();
        let out = temp_dir("now-out");
        build(&vault, &out, &[]).expect("build with a published now page");
        out
    })
}

fn footer(html: &str) -> &str {
    html.split("<footer class=\"site-footer\">").nth(1).unwrap_or("")
}

#[test]
fn now_page_credits_derek_sivers_and_nownownow() {
    let md = read(&content_dir().join("now.md"));
    assert!(md.contains("Derek Sivers"), "now.md doesn't credit Derek Sivers");
    assert!(
        md.contains("(https://nownownow.com/about)"),
        "now.md doesn't link nownownow.com/about"
    );
    let html = read(&published_now().join("now/index.html"));
    assert!(html.contains(r#"href="https://nownownow.com/about""#));
}

#[test]
fn now_page_says_when_it_was_updated() {
    let html = read(&published_now().join("now/index.html"));
    let p = html
        .split("<p class=\"now-updated\">")
        .nth(1)
        .expect("no Updated line on the now page");
    assert!(
        p.starts_with("Updated <time datetime=\""),
        "Updated line has no <time>: {p:.80}"
    );
}

#[test]
fn every_page_footer_links_a_published_now_page() {
    let out = published_now();
    let missing: Vec<String> = files_with_ext(out, "html")
        .iter()
        .filter(|p| !footer(&read(p)).contains(r#"<a href="/now/">Now</a>"#))
        .map(|p| rel(p, out))
        .collect();
    assert!(
        missing.is_empty(),
        "pages without a Now footer link:\n{}",
        missing.join("\n")
    );
}

#[test]
fn footer_links_now_only_when_the_page_is_built() {
    let built = site().join("now/index.html").exists();
    for (p, h) in html_files() {
        assert_eq!(
            footer(h).contains("/now/"),
            built,
            "{}: Now footer link doesn't match whether /now/ is built",
            rel(p, site())
        );
    }
}

#[test]
fn a_now_page_without_an_updated_date_fails_the_build() {
    let vault = vault_copy("now-undated");
    fs::write(vault.join("now.md"), "---\ntitle: Now\n---\n\nBusy.\n").unwrap();
    let err = build(&vault, &temp_dir("now-undated-out"), &[]).expect_err("build should fail");
    assert!(err.contains("now.md"), "unexpected error: {err}");
}

#[test]
fn no_published_page_has_a_placeholder() {
    let found: Vec<String> = html_files()
        .iter()
        .filter(|(_, h)| h.contains(PLACEHOLDER))
        .map(|(p, _)| rel(p, Path::new(site())))
        .collect();
    assert!(
        found.is_empty(),
        "{PLACEHOLDER} left on published pages; fill it in or keep the note a draft:\n{}",
        found.join("\n")
    );
}
