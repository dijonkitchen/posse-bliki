//! The blogroll page and its OPML export (spec/feeds.md, ADR 2026-10-25-blogroll).

use std::fs;

use crate::support::html::Element;
use crate::support::{build, content_dir, html_files, read, rel, site, temp_dir, vault_copy, xml};

fn outlines() -> Vec<Element> {
    let root = xml::parse(&read(&site().join("blogroll.opml"))).expect("blogroll.opml is well-formed XML");
    assert_eq!(root.name, "opml");
    assert_eq!(root.attr("version"), Some("2.0"));
    let head = root.child("head").expect("no <head>");
    assert!(!head.child("title").expect("no <title>").text().is_empty());
    let body = root.child("body").expect("no <body>");
    body.elements().filter(|e| e.name == "outline").cloned().collect()
}

#[test]
fn opml_has_an_outline_per_listed_site() {
    let md = read(&content_dir().join("blogroll.md"));
    let listed = md.lines().filter(|l| l.starts_with("- [")).count();
    let outlines = outlines();
    assert!(listed > 0, "blogroll.md lists no sites");
    assert_eq!(outlines.len(), listed, "one outline per list item in blogroll.md");
    let web = |u: Option<&str>| u.is_some_and(|u| u.starts_with("https://") || u.starts_with("http://"));
    for o in &outlines {
        assert!(!o.attr("text").unwrap_or("").is_empty(), "outline without text");
        match o.attr("type") {
            Some("rss") => {
                assert!(web(o.attr("xmlUrl")), "rss outline needs an absolute xmlUrl");
                assert!(web(o.attr("htmlUrl")), "rss outline needs an absolute htmlUrl");
            }
            Some("link") => assert!(web(o.attr("url")), "link outline needs an absolute url"),
            other => panic!("unexpected outline type {other:?}"),
        }
    }
}

#[test]
fn blogroll_page_points_at_its_opml() {
    let html = read(&site().join("blogroll/index.html"));
    assert!(
        html.contains(r#"<link rel="alternate" type="text/x-opml""#),
        "no OPML <link> in the blogroll page head"
    );
    assert!(
        html.contains(r#"href="/blogroll.opml""#),
        "no visible link to the OPML file"
    );
}

#[test]
fn every_page_footer_links_the_blogroll() {
    let missing: Vec<String> = html_files()
        .iter()
        .filter(|(_, h)| {
            let footer = h.split("<footer class=\"site-footer\">").nth(1).unwrap_or("");
            !footer.contains(r#"<a href="/blogroll/">Blogroll</a>"#)
        })
        .map(|(p, _)| rel(p, site()))
        .collect();
    assert!(
        missing.is_empty(),
        "pages without a Blogroll footer link:\n{}",
        missing.join("\n")
    );
}

#[test]
fn no_blogroll_page_means_no_opml_and_no_link() {
    let vault = vault_copy("no-blogroll");
    fs::remove_file(vault.join("blogroll.md")).unwrap();
    let colophon = vault.join("colophon.md");
    let kept: Vec<String> = read(&colophon)
        .lines()
        .filter(|l| !l.contains("blogroll.md"))
        .map(String::from)
        .collect();
    fs::write(&colophon, kept.join("\n")).unwrap();
    let out = temp_dir("no-blogroll-out");
    build(&vault, &out, &[]).expect("build without a blogroll");
    assert!(
        !out.join("blogroll.opml").exists(),
        "blogroll.opml without a blogroll page"
    );
    assert!(
        !read(&out.join("index.html")).contains("/blogroll/"),
        "footer links a blogroll that doesn't exist"
    );
}

#[test]
fn an_item_without_a_site_link_fails_the_build() {
    let vault = vault_copy("bad-blogroll");
    fs::write(
        vault.join("blogroll.md"),
        "---\ntitle: Blogroll\n---\n\n- [A](https://a.example/)\n- just a name\n",
    )
    .unwrap();
    let err = build(&vault, &temp_dir("bad-blogroll-out"), &[]).expect_err("build should fail");
    assert!(err.contains("blogroll.md: item 2"), "unexpected error: {err}");
}
