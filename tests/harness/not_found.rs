//! The 404 page (spec/output-contract.md, "Error pages";
//! ADR 2026-11-06-not-found-page).

use crate::support::{attr_values, read, site};

fn not_found() -> String {
    read(&site().join("404.html"))
}

fn main_content(html: &str) -> &str {
    html.split("<main id=\"main\">")
        .nth(1)
        .and_then(|rest| rest.split("</main>").next())
        .expect("404.html has no <main>")
}

#[test]
fn every_link_works_at_any_depth() {
    // Pages serves the same file for /nope and /notes/old/post/, so a
    // relative link would point somewhere different from each.
    let html = not_found();
    for url in attr_values(&html, "href").into_iter().chain(attr_values(&html, "src")) {
        assert!(
            url.starts_with('/') || url.starts_with('#') || url.starts_with("https://"),
            "404.html links {url:?}, which breaks below the site root"
        );
    }
}

#[test]
fn points_lost_readers_home_to_the_notes_and_to_follow() {
    let html = not_found();
    let body = main_content(&html);
    assert!(
        body.contains("<h1>Page not found</h1>"),
        "404.html has no heading:\n{body}"
    );
    for href in ["/", "/notes/", "/follow/"] {
        assert!(
            body.contains(&format!("<a href=\"{href}\">")),
            "404.html doesn't link {href} in its text:\n{body}"
        );
    }
}

#[test]
fn stays_out_of_search() {
    let html = not_found();
    assert!(
        html.contains("<meta name=\"robots\" content=\"noindex\">"),
        "404.html should be noindex"
    );
    assert!(!html.contains("rel=\"canonical\""), "404.html should have no canonical");
    assert!(
        !read(&site().join("sitemap.xml")).contains("404.html"),
        "404.html is listed in the sitemap"
    );
}

#[test]
fn worker_never_keeps_a_not_found() {
    // Offline, a kept 404 would answer for its address in place of the
    // offline page, telling the reader a page is missing when it may since
    // have been published.
    let js = read(&site().join("sw.js"));
    assert!(
        js.contains("if (response.status === 200) {"),
        "sw.js should keep 200 responses only"
    );
}
