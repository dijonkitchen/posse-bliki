//! Bridgy links on posts, received mentions rendered at build time, and the
//! list of webmentions to send for new posts (spec/indieweb.md, ADR 2026-10-11-webmentions-at-deploy-time).

use std::path::PathBuf;
use std::sync::OnceLock;

use posse_bliki_build::json::{self, Json};

use crate::support::mf2::{self, Value};
use crate::support::{
    bliki, build, config_str, content_dir, html, post_html_files, read, site, syndicate_to, temp_dir,
};

const POST: &str = "notes/architecture";

#[test]
fn every_post_links_to_each_syndication_target() {
    let targets = syndicate_to();
    assert!(!targets.is_empty(), "no syndication targets configured");
    for f in post_html_files() {
        let text = read(&f);
        for (t, _) in &targets {
            assert!(
                text.contains(&format!("<a href=\"{t}\" hidden></a>")),
                "{}: no link to {t}",
                f.display()
            );
        }
    }
}

#[test]
fn no_responses_section_without_webmentions() {
    for f in post_html_files() {
        assert!(!read(&f).contains("class=\"webmentions\""), "{}", f.display());
    }
}

fn mentioned_site() -> &'static PathBuf {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        // Targets flip the www. prefix and use http:// on purpose: they must still match.
        let host = config_str("base_url").split_once("://").unwrap().1.trim_end_matches('/');
        let other = match host.strip_prefix("www.") {
            Some(bare) => bare.to_string(),
            None => format!("www.{host}"),
        };
        let target = format!("http://{other}/{POST}/");
        let children = [
            format!(r#"{{"wm-property": "like-of", "wm-target": "{target}", "url": "https://m.example/likes/1", "author": {{"name": "Ada", "url": "https://ada.example/"}}}}"#),
            format!(r#"{{"wm-property": "repost-of", "wm-target": "{target}", "url": "https://m.example/boosts/1", "author": {{"name": "Bo", "url": "https://bo.example/"}}}}"#),
            format!(r#"{{"wm-property": "in-reply-to", "wm-target": "{target}", "url": "https://m.example/replies/1", "author": {{"name": "Cy <script>", "url": "javascript:alert(1)"}}, "published": "2026-10-02T10:00:00Z", "content": {{"text": "Nice <b>post</b>"}}}}"#),
            format!(r#"{{"wm-property": "in-reply-to", "wm-target": "{target}", "url": "https://m.example/replies/2", "wm-private": true, "content": {{"text": "secret"}}}}"#),
            format!(r#"{{"wm-property": "like-of", "wm-target": "{target}", "url": "javascript:alert(2)"}}"#),
        ];
        let dir = temp_dir("wm");
        let jf2 = dir.join("webmentions.json");
        std::fs::write(&jf2, format!(r#"{{"type": "feed", "children": [{}]}}"#, children.join(", "))).unwrap();
        let out = dir.join("public");
        build(&content_dir(), &out, &["--webmentions", jf2.to_str().unwrap()]).expect("build with webmentions");
        out
    })
}

fn post_page() -> String {
    read(&mentioned_site().join(POST).join("index.html"))
}

#[test]
fn mentions_render_on_their_post() {
    let html = post_page();
    assert!(html.contains("<section class=\"webmentions\">"));
    assert!(html.contains("Liked by 1") && html.contains("Reposted by 1"));
    assert!(html.contains("Nice &lt;b&gt;post&lt;/b&gt;"));
    assert!(html.contains("Cy &lt;script&gt;"));
    assert!(!html.contains("javascript:"));
    assert!(!html.contains("secret"));
}

#[test]
fn mentions_are_mf2_responses_of_the_entry() {
    let doc = mf2::parse(&html::parse(&post_page()).expect("valid HTML"));
    let entry = doc.items_of_type("h-entry")[0];
    assert_eq!(entry.prop("like").len(), 1);
    assert_eq!(entry.prop("repost").len(), 1);
    let comments = entry.prop("comment");
    assert_eq!(comments.len(), 1);
    match &comments[0] {
        Value::Item(cite) => {
            assert!(cite.has_type("h-cite"));
            assert_eq!(cite.texts("url"), ["https://m.example/replies/1"]);
        }
        other => panic!("comment is not an h-cite: {other:?}"),
    }
}

#[test]
fn mentions_only_on_their_target() {
    let notes = mentioned_site().join("notes");
    for f in crate::support::files_with_ext(&notes, "html") {
        if !f.parent().unwrap().ends_with(POST) {
            assert!(!read(&f).contains("class=\"webmentions\""), "{}", f.display());
        }
    }
}

#[test]
fn bad_webmentions_file_fails_the_build() {
    let dir = temp_dir("wm-bad");
    let bad = dir.join("wm.json");
    std::fs::write(&bad, "{not json").unwrap();
    assert!(build(
        &content_dir(),
        &dir.join("out"),
        &["--webmentions", bad.to_str().unwrap()]
    )
    .is_err());
}

fn pending(previous: &std::path::Path) -> std::process::Output {
    bliki(&[
        "--out",
        site().to_str().unwrap(),
        "--pending-webmentions",
        previous.to_str().unwrap(),
    ])
}

#[test]
fn pending_lists_new_posts_per_target() {
    let feed = json::parse(&read(&site().join("feed.json"))).unwrap();
    let items = feed.get("items").unwrap().as_arr();
    let new = items[0].str_at("url").unwrap();
    // The previous deploy's feed: the same feed without its newest item.
    let Json::Obj(fields) = &feed else {
        panic!("feed is not an object")
    };
    let previous: Vec<(String, Json)> = fields
        .iter()
        .map(|(k, v)| match (k.as_str(), v) {
            ("items", Json::Arr(items)) => (k.clone(), Json::Arr(items[1..].to_vec())),
            _ => (k.clone(), v.clone()),
        })
        .collect();
    let prev = temp_dir("pending").join("previous.json");
    std::fs::write(&prev, json::dumps(&Json::Obj(previous))).unwrap();
    let proc = pending(&prev);
    assert!(proc.status.success());
    let expected: Vec<String> = syndicate_to()
        .iter()
        .map(|(t, ep)| format!("{ep}\t{new}\t{t}"))
        .collect();
    assert_eq!(
        String::from_utf8(proc.stdout).unwrap().lines().collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn pending_is_empty_when_nothing_is_new() {
    let proc = pending(&site().join("feed.json"));
    assert!(proc.status.success());
    assert!(proc.stdout.is_empty());
}

#[test]
fn pending_fails_on_a_feed_that_is_not_json() {
    let prev = temp_dir("pending-bad").join("previous.json");
    std::fs::write(&prev, "<html>Not found</html>").unwrap();
    let proc = pending(&prev);
    assert!(!proc.status.success() && proc.stdout.is_empty());
}
