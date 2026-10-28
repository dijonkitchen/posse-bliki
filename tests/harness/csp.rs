//! Content Security Policy (spec/output-contract.md, ADR 2026-10-28-csp-meta-tag):
//! every page carries the policy, and nothing the build emits needs more
//! than it allows, so the policy can't quietly break a page.

use std::fs;

use crate::support::{files_with_ext, html, html_files, rel, site};

const CSP: &str = "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; \
script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; manifest-src 'self'; connect-src 'self'; \
base-uri 'none'; form-action 'none'\">";

/// A path on this site, which `'self'` allows: `/x`, never `//host/x`.
fn same_origin(url: &str) -> bool {
    url.starts_with('/') && !url.starts_with("//")
}

#[test]
fn every_page_sets_the_policy_first() {
    // A policy only covers what comes after it, so it sits right after the
    // charset, ahead of every stylesheet, font and script.
    let first = format!("<head>\n  <meta charset=\"utf-8\">\n  {CSP}\n");
    for (p, text) in html_files() {
        assert!(
            text.contains(&first),
            "{}: CSP meta missing or not first in <head>",
            rel(p, site())
        );
    }
}

#[test]
fn pages_load_only_same_origin_files() {
    let mut bad = Vec::new();
    for (p, text) in html_files() {
        let root = html::parse(text).expect("parse");
        for el in root.descendants() {
            let at = |what: &str| format!("{}: <{}> {what}", rel(p, site()), el.name);
            match el.name.as_str() {
                "style" | "iframe" | "object" | "embed" | "video" | "audio" | "frame" => bad.push(at("is blocked")),
                "script" if el.attr("src").is_none_or(|s| !same_origin(s)) => bad.push(at("isn't a local file")),
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
