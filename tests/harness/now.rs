//! The now page, after Derek Sivers's (spec/content-schema.md, ADR 2026-11-01-now-page,
//! ADR 2026-11-05-now-in-the-header).

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

/// The hrefs of the header's site nav, in order.
fn nav_hrefs(html: &str) -> Vec<String> {
    let nav = html
        .split("<nav aria-label=\"Site\">")
        .nth(1)
        .and_then(|rest| rest.split("</nav>").next())
        .unwrap_or("");
    nav.split("href=\"")
        .skip(1)
        .filter_map(|h| h.split('"').next())
        .map(String::from)
        .collect()
}

#[test]
fn every_page_header_reads_notes_now_about() {
    let out = published_now();
    let wrong: Vec<String> = files_with_ext(out, "html")
        .iter()
        .filter(|p| nav_hrefs(&read(p)) != ["/notes/", "/now/", "/about/"])
        .map(|p| rel(p, out))
        .collect();
    assert!(
        wrong.is_empty(),
        "pages whose header nav isn't Notes, Now, About:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn every_page_footer_links_the_colophon() {
    let missing: Vec<String> = html_files()
        .iter()
        .filter(|(_, h)| !footer(h).contains(r#"<a href="/colophon/">Colophon</a>"#))
        .map(|(p, _)| rel(p, site()))
        .collect();
    assert!(
        missing.is_empty(),
        "pages without a Colophon footer link:\n{}",
        missing.join("\n")
    );
}

#[test]
fn no_now_page_means_no_now_link() {
    let vault = vault_copy("no-now");
    fs::remove_file(vault.join("now.md")).unwrap();
    let out = temp_dir("no-now-out");
    build(&vault, &out, &[]).expect("build without a now page");
    let home = read(&out.join("index.html"));
    assert_eq!(nav_hrefs(&home), ["/notes/", "/about/"]);
    assert!(!home.contains("/now/"), "links a now page that doesn't exist");
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
