//! Search (ADR 2026-11-21-search-page): `/search/` is a form that
//! `/search.js` answers in the browser from `/search.json`, plain text of
//! every note and page. Nothing is sent anywhere; with JS off the form stays
//! disabled and the page points to Notes.

use std::collections::BTreeSet;
use std::fs;

use posse_bliki_build::json::{self, Json};

use crate::support::{build, config_str, html, read, repo, site, temp_dir, vault_copy};

fn script() -> String {
    fs::read_to_string(repo().join("build/static/search.js")).expect("read search.js")
}

/// `(url, title, text)` of each entry in `search.json`, in order.
fn index(out: &std::path::Path) -> Vec<(String, String, String)> {
    let parsed = json::parse(&read(&out.join("search.json"))).expect("search.json is JSON");
    let Json::Arr(entries) = parsed else {
        panic!("search.json is not an array");
    };
    entries
        .iter()
        .map(|e| {
            let field = |k: &str| e.str_at(k).unwrap_or_else(|| panic!("entry without {k}")).to_string();
            (field("url"), field("title"), field("text"))
        })
        .collect()
}

#[test]
fn search_page_form_waits_for_its_script() {
    let page = read(&site().join("search/index.html"));
    let doc = html::parse(&page).expect("search page parses");
    let all = doc.descendants();
    let by_id = |id: &str| {
        *all.iter()
            .find(|e| e.attr("id") == Some(id))
            .unwrap_or_else(|| panic!("no #{id}"))
    };
    let form = by_id("search");
    assert_eq!((form.name.as_str(), form.attr("role")), ("form", Some("search")));
    let input = by_id("search-box");
    assert_eq!(
        (input.attr("type"), input.attr("name")),
        (Some("search"), Some("q")),
        "the box is a search input sending q"
    );
    assert!(
        all.iter()
            .any(|e| e.name == "label" && e.attr("for") == Some("search-box")),
        "the box has no label"
    );
    // Disabled until the script runs, so with JS off it's not a box that
    // does nothing; the script never submits it, so form-action stays 'none'.
    let controls: Vec<_> = form
        .descendants()
        .into_iter()
        .filter(|e| e.name == "input" || e.name == "button")
        .collect();
    assert_eq!(controls.len(), 2, "the form should hold the box and its button");
    for control in controls {
        assert!(control.attr("disabled").is_some(), "<{}> isn't disabled", control.name);
    }
    let status = by_id("search-status");
    assert_eq!(status.attr("role"), Some("status"), "results aren't announced");
    let noscript = status.child("noscript").expect("no note for readers without JS");
    assert!(
        noscript.descendants().iter().any(|e| e.attr("href") == Some("/notes/")),
        "the note doesn't point to Notes"
    );
    assert!(
        page.contains("<script src=\"/search.js\" defer></script>"),
        "the page doesn't load /search.js deferred"
    );
}

#[test]
fn search_page_is_canonical_and_described() {
    let page = read(&site().join("search/index.html"));
    let base = config_str("base_url").trim_end_matches('/');
    assert!(page.contains(&format!("<link rel=\"canonical\" href=\"{base}/search/\">")));
    assert!(page.contains("<meta name=\"description\" content=\"Search every note and page on "));
}

#[test]
fn search_script_ships_unchanged() {
    assert_eq!(
        read(&site().join("search.js")),
        script(),
        "search.js differs from build/static/search.js"
    );
}

#[test]
fn index_has_every_note_in_notes_order_then_every_page() {
    let out = site();
    let base = config_str("base_url").trim_end_matches('/');
    let urls: Vec<String> = index(out).into_iter().map(|(url, ..)| url).collect();
    let notes_page = read(&out.join("notes/index.html"));
    let notes: Vec<String> = notes_page
        .split("<a class=\"u-url p-name\" href=\"")
        .skip(1)
        .map(|rest| format!("{base}{}", rest.split('"').next().unwrap()))
        .collect();
    assert!(!notes.is_empty(), "no notes listed on /notes/");
    assert_eq!(
        urls[..notes.len()],
        notes[..],
        "notes aren't first, in the order of /notes/"
    );
    // The sitemap lists every built note and page: the index has each of
    // them but home and /notes/, and nothing else.
    let sitemap: BTreeSet<String> = read(&out.join("sitemap.xml"))
        .split("<loc>")
        .skip(1)
        .map(|rest| rest.split("</loc>").next().unwrap().to_string())
        .filter(|u| *u != format!("{base}/") && *u != format!("{base}/notes/"))
        .collect();
    let indexed: BTreeSet<String> = urls.iter().cloned().collect();
    assert_eq!(indexed.len(), urls.len(), "an entry is in search.json twice");
    assert_eq!(indexed, sitemap, "search.json and the sitemap list different pages");
}

#[test]
fn index_is_plain_text() {
    let vault = vault_copy("search-text");
    fs::write(
        vault.join("notes/search-fixture.md"),
        "---\ntitle: Fish & chips\npublished: true\ncreated_at: 2026-05-09\n---\n\nA **bold** claim, a [link](raw.md)\nand `a < b`.\n\n- one\n- two\n",
    )
    .unwrap();
    let out = temp_dir("search-text-out");
    build(&vault, &out, &[]).expect("build with the fixture");
    let entries = index(&out);
    let (_, title, text) = entries
        .iter()
        .find(|(url, ..)| url.ends_with("/notes/search-fixture/"))
        .expect("fixture not in search.json");
    assert_eq!(title, "Fish & chips");
    assert_eq!(text, "A bold claim, a link and a < b. one two");
}

#[test]
fn unpublished_and_scheduled_notes_stay_out_of_the_index() {
    let vault = vault_copy("search-hidden");
    fs::write(
        vault.join("notes/search-draft.md"),
        "---\ntitle: Draft\ncreated_at: 2026-05-09\n---\n\nnot for the index\n",
    )
    .unwrap();
    fs::write(
        vault.join("notes/search-later.md"),
        "---\ntitle: Later\npublished: true\ncreated_at: 2099-01-01\n---\n\nnot for the index yet\n",
    )
    .unwrap();
    let out = temp_dir("search-hidden-out");
    build(&vault, &out, &[]).expect("build with the hidden notes");
    let text = read(&out.join("search.json"));
    assert!(!text.contains("not for the index"), "a hidden note is in search.json");
}

#[test]
fn search_script_writes_text_never_html() {
    // Note text becomes text nodes, so nothing in a note can turn into
    // markup or script on the search page.
    let js = script();
    for sink in [
        "innerHTML",
        "outerHTML",
        "insertAdjacentHTML",
        "document.write",
        "DOMParser",
        "eval(",
    ] {
        assert!(!js.contains(sink), "search.js uses {sink}");
    }
}

#[test]
fn search_script_stays_on_the_device_and_on_the_page() {
    let js = script();
    // Only search.json, beside the script, so it works under a sub-path
    // and asks no other host.
    assert_eq!(js.matches("fetch(").count(), 1, "search.js fetches more than one thing");
    assert!(
        js.contains("new URL(\"search.json\", document.currentScript.src)") && js.contains("fetch(index)"),
        "search.js should fetch search.json beside itself"
    );
    // The form never submits, so the CSP's form-action 'none' holds.
    assert!(js.contains("event.preventDefault();"), "search.js lets the form submit");
}

#[test]
fn search_script_uses_modern_syntax() {
    // Like the style picker: no `var`, helpers as `const` arrows defined
    // before their first use (ADR 2026-10-28-modern-js-style-picker).
    let js = script();
    let code: Vec<&str> = js.lines().filter(|l| !l.trim_start().starts_with("//")).collect();
    assert!(
        code.first().is_some_and(|l| l.starts_with("(() => {")),
        "search.js isn't one arrow IIFE"
    );
    assert!(js.contains("\"use strict\";"), "search.js isn't strict");
    assert!(
        !code.iter().any(|l| l.contains("var ") || l.contains("function")),
        "search.js should use const/let and arrow functions"
    );
    for helper in ["fold", "words", "find", "load", "snippet", "show", "search"] {
        let defined = js
            .find(&format!("const {helper} = "))
            .unwrap_or_else(|| panic!("no const {helper}"));
        assert!(
            !js[..defined].contains(&format!("{helper}(")),
            "{helper} is called above its definition"
        );
    }
}
