//! Offline reading (ADR 2026-10-29-offline-reading): a service worker keeps
//! pages as they're read and serves them when the network is gone.

use std::fs;

use crate::support::{read, repo, site};

fn worker() -> String {
    read(&site().join("sw.js"))
}

/// The paths in `sw.js`'s `SHELL` list, as written.
fn shell(js: &str) -> Vec<String> {
    let start = js.find("const SHELL = [").expect("sw.js has no SHELL list");
    let list = &js[start..start + js[start..].find("];").expect("unclosed SHELL list")];
    list.split('"').skip(1).step_by(2).map(String::from).collect()
}

#[test]
fn worker_ships_unchanged() {
    let source = fs::read_to_string(repo().join("build/static/sw.js")).expect("read source");
    assert_eq!(worker(), source, "sw.js differs from build/static/sw.js");
}

#[test]
fn shell_files_exist() {
    // A missing file fails the whole install, so offline reading would
    // silently never start.
    let paths = shell(&worker());
    assert!(
        paths.contains(&"offline.html".to_string()),
        "offline page not kept on install"
    );
    for path in paths {
        assert!(
            !path.starts_with('/'),
            "{path}: shell paths stay relative for sub-paths"
        );
        let file = if path == "./" {
            "index.html".to_string()
        } else {
            path.clone()
        };
        assert!(
            site().join(&file).is_file(),
            "sw.js keeps {path}, which the build doesn't emit"
        );
    }
}

#[test]
fn worker_is_network_first_for_own_gets() {
    // Online readers always get the live page; only the site's own GETs
    // are kept.
    let js = worker();
    assert!(
        js.contains("request.method !== \"GET\""),
        "worker should ignore non-GET requests"
    );
    assert!(
        js.contains(".origin !== location.origin"),
        "worker should ignore other hosts"
    );
    let network = js.find("await fetch(request)").expect("worker never asks the network");
    let fallback = js
        .find("return fromCache(request)")
        .expect("worker never falls back to its copy");
    assert!(network < fallback, "worker should try the network before its copy");
}

#[test]
fn style_picker_starts_the_worker() {
    let js = read(&site().join("style-picker.js"));
    assert!(
        js.contains("new URL(\"sw.js\", document.currentScript.src)"),
        "style-picker.js should register sw.js relative to itself"
    );
    assert!(
        js.contains("navigator.serviceWorker.register("),
        "style-picker.js never registers sw.js"
    );
}

#[test]
fn offline_page_is_not_indexed() {
    let html = read(&site().join("offline.html"));
    assert!(
        html.contains("<meta name=\"robots\" content=\"noindex\">"),
        "offline.html should be noindex"
    );
    assert!(
        !read(&site().join("sitemap.xml")).contains("offline.html"),
        "offline.html is listed in the sitemap"
    );
}
