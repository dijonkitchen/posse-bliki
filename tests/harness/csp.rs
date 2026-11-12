//! Content Security Policy (spec/output-contract.md, ADR 2026-10-28-csp-meta-tag):
//! every page carries the policy, and nothing the build emits needs more
//! than it allows, so the policy can't quietly break a page. The house style
//! and the style picker are inline, each allowed by its hash
//! (ADR 2026-11-11-inline-house-style-and-script).

use std::fs;

use posse_bliki_build::sha256::csp_source;

use crate::support::{files_with_ext, html, html_files, rel, site};

/// A path on this site, which `'self'` allows: `/x`, never `//host/x`.
fn same_origin(url: &str) -> bool {
    url.starts_with('/') && !url.starts_with("//")
}

/// The text of the first `<name ...>...</name>` element in `html`, exactly as
/// the browser hashes it: everything between the start tag's `>` and the end tag.
pub(crate) fn inline_text<'a>(html: &'a str, name: &str) -> &'a str {
    let start = html.find(&format!("<{name}")).unwrap_or_else(|| panic!("no <{name}>"));
    let open_end = start + html[start..].find('>').expect("unclosed start tag") + 1;
    let close = open_end + html[open_end..].find(&format!("</{name}>")).expect("no end tag");
    &html[open_end..close]
}

/// The policy a page must carry: the site's own files, plus its inline
/// script and style by hash.
pub(crate) fn expected_policy(html: &str) -> String {
    format!(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; \
script-src 'self' {}; style-src 'self' {}; img-src 'self'; font-src 'self'; manifest-src 'self'; \
connect-src 'self'; base-uri 'none'; form-action 'none'\">",
        csp_source(inline_text(html, "script")),
        csp_source(inline_text(html, "style")),
    )
}

#[test]
fn every_page_sets_the_policy_first() {
    // A policy only covers what comes after it, so it sits right after the
    // charset, ahead of every stylesheet, font and script. Its hashes are
    // those of the page's own inline blocks, or the browser would refuse them.
    for (p, text) in html_files() {
        let first = format!("<head>\n  <meta charset=\"utf-8\">\n  {}\n", expected_policy(text));
        assert!(
            text.contains(&first),
            "{}: CSP meta missing, not first in <head>, or its hashes don't match the inline blocks",
            rel(p, site())
        );
    }
}

#[test]
fn pages_load_only_same_origin_files() {
    let mut bad = Vec::new();
    for (p, text) in html_files() {
        let root = html::parse(text).expect("parse");
        let (mut inline_styles, mut inline_scripts) = (0, 0);
        for el in root.descendants() {
            let at = |what: &str| format!("{}: <{}> {what}", rel(p, site()), el.name);
            match el.name.as_str() {
                "iframe" | "object" | "embed" | "video" | "audio" | "frame" => bad.push(at("is blocked")),
                // Inline, allowed by hash: only the house style and the picker.
                "style" => inline_styles += 1,
                "script" if el.attr("src").is_none() => inline_scripts += 1,
                "script" if el.attr("src").is_some_and(|s| !same_origin(s)) => bad.push(at("isn't a local file")),
                "img" | "source" => {
                    for url in [el.attr("src"), el.attr("srcset")].into_iter().flatten() {
                        if !url.split(',').all(|u| same_origin(u.trim())) {
                            bad.push(at(&format!("loads {url}")));
                        }
                    }
                }
                "link" => {
                    let rel_ = el.attr("rel").unwrap_or("");
                    let loads = ["stylesheet", "icon", "apple-touch-icon", "manifest", "preload"];
                    if rel_.split_ascii_whitespace().any(|r| loads.contains(&r))
                        && !el.attr("href").is_some_and(same_origin)
                    {
                        bad.push(at(&format!("loads {}", el.attr("href").unwrap_or(""))));
                    }
                }
                _ => {}
            }
            for (k, _) in &el.attrs {
                if k == "style" || k.starts_with("on") {
                    bad.push(at(&format!("has inline {k}=")));
                }
            }
        }
        if (inline_styles, inline_scripts) != (1, 1) {
            bad.push(format!(
                "{}: {inline_styles} <style> and {inline_scripts} inline <script>, but the policy allows one of each",
                rel(p, site())
            ));
        }
    }
    assert!(bad.is_empty(), "the CSP would block these:\n{}", bad.join("\n"));
}

#[test]
fn stylesheets_load_only_same_origin_files() {
    let mut bad = Vec::new();
    for p in files_with_ext(site(), "css") {
        let css = fs::read_to_string(&p).unwrap();
        for (i, _) in css.match_indices("url(") {
            let url = css[i + 4..].split(')').next().unwrap().trim_matches(['"', '\'', ' ']);
            if url.contains(':') || url.starts_with("//") {
                bad.push(format!("{}: url({url})", rel(&p, site())));
            }
        }
        if css.contains("@import") {
            bad.push(format!("{}: @import", rel(&p, site())));
        }
    }
    assert!(bad.is_empty(), "the CSP would block these:\n{}", bad.join("\n"));
}
