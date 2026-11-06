//! h-entry on every post, h-card on home, h-feed on lists
//! (spec/output-contract.md).

use std::path::Path;

use crate::support::mf2::{self, Value};
use crate::support::{html, post_html_files, read, rel, site};

fn parsed(path: &Path) -> mf2::Parsed {
    mf2::parse(&html::parse(&read(path)).expect("valid HTML"))
}

/// Bylines show only the nickname; parsers still get the full name.
fn full_name() -> &'static str {
    crate::support::config()
        .get("author")
        .and_then(|a| a.str_at("name"))
        .expect("config has no author.name")
}

#[test]
fn home_has_h_card() {
    let home = parsed(&site().join("index.html"));
    let cards = home.items_of_type("h-card");
    let card = cards.first().expect("home page has no h-card");
    assert!(!card.prop("name").is_empty(), "h-card.p-name missing");
    // The page shows only the nickname; parsers still get the full name.
    let full_name = full_name();
    assert!(
        card.prop("name")
            .iter()
            .any(|v| matches!(v, Value::Text(t) if t == full_name)),
        "h-card.p-name is not the author's full name: {:?}",
        card.prop("name")
    );
    assert!(!card.prop("url").is_empty(), "h-card.u-url missing");
}

/// indiewebify.me looks for a short bio as the h-card's `p-note`
/// (ADR 2026-11-06-h-card-bio-note).
#[test]
fn home_h_card_has_a_bio_note() {
    let home = parsed(&site().join("index.html"));
    let cards = home.items_of_type("h-card");
    let card = cards.first().expect("home page has no h-card");
    let note = crate::support::config()
        .get("author")
        .and_then(|a| a.str_at("note"))
        .expect("config has no author.note");
    assert!(!note.trim().is_empty(), "config author.note is empty");
    assert!(
        card.prop("note")
            .iter()
            .any(|v| matches!(v, Value::Text(t) if t == note)),
        "h-card.p-note is not the configured bio: {:?}",
        card.prop("note")
    );
}

/// The h-card's photo is the author's avatar in readers, webmention
/// displays and Bridgy Fed (ADR 2026-11-06-h-card-photo).
#[test]
fn home_h_card_has_a_photo_the_site_serves() {
    let home = parsed(&site().join("index.html"));
    let cards = home.items_of_type("h-card");
    let card = cards.first().expect("home page has no h-card");
    let author = crate::support::config().get("author").expect("config has no author");
    let photo = author.str_at("photo").expect("config has no author.photo");
    let url = format!("{}{photo}", crate::support::config_str("base_url"));
    assert!(
        card.prop("photo")
            .iter()
            .any(|v| matches!(v, Value::Text(t) if *t == url)),
        "h-card.u-photo is not {url}: {:?}",
        card.prop("photo")
    );
    let file = site().join(photo.trim_start_matches('/'));
    let bytes = std::fs::read(&file).unwrap_or_else(|_| panic!("{photo} is not in the built site"));
    let (w, h) = posse_bliki_build::image::size(&bytes).unwrap_or_else(|| panic!("{photo} is not an image"));
    assert_eq!(
        w, h,
        "{photo} is {w}x{h}; avatars are cropped square, so make it square"
    );
}

/// The bio page opens with the same photo, sized so the page doesn't
/// jump when it loads (ADR 2026-11-06-h-card-photo).
#[test]
fn bio_page_opens_with_the_photo() {
    let photo = crate::support::config()
        .get("author")
        .and_then(|a| a.str_at("photo"))
        .expect("config has no author.photo");
    let html = read(&site().join("bio/index.html"));
    let img = html
        .split("<img ")
        .nth(1)
        .map(|t| &t[..t.find('>').unwrap_or(t.len())])
        .expect("bio page has no image");
    assert!(
        img.contains(&format!("src=\"{photo}\"")),
        "bio's first image isn't {photo}: {img}"
    );
    assert!(img.contains("alt=\"JC\""), "bio photo needs alt text: {img}");
    assert!(
        img.contains("width=\"") && img.contains("height=\""),
        "bio photo needs its size: {img}"
    );
    assert!(
        !img.contains("loading=\"lazy\""),
        "bio photo is on the first screen: {img}"
    );
}

#[test]
fn home_has_h_feed() {
    assert!(
        !parsed(&site().join("index.html")).items_of_type("h-feed").is_empty(),
        "home page has no h-feed"
    );
}

#[test]
fn every_post_has_h_entry() {
    let posts = post_html_files();
    assert!(!posts.is_empty(), "no post HTML files were emitted");
    for f in posts {
        assert!(
            !parsed(&f).items_of_type("h-entry").is_empty(),
            "{}: no h-entry found",
            rel(&f, site())
        );
    }
}

#[test]
fn h_entry_has_required_properties() {
    for f in post_html_files() {
        let doc = parsed(&f);
        let entry = doc.items_of_type("h-entry")[0];
        for prop in ["name", "published", "content", "url", "author"] {
            assert!(
                !entry.prop(prop).is_empty(),
                "{}: h-entry missing p-/u-/dt-/e- {prop}",
                rel(&f, site())
            );
        }
    }
}

#[test]
fn h_entry_author_is_h_card() {
    for f in post_html_files() {
        let doc = parsed(&f);
        let entry = doc.items_of_type("h-entry")[0];
        match entry.prop("author").first() {
            Some(Value::Item(card)) if card.has_type("h-card") => assert!(
                card.prop("name")
                    .iter()
                    .any(|v| matches!(v, Value::Text(t) if t == full_name())),
                "{}: byline h-card.p-name is not the author's full name",
                rel(&f, site())
            ),
            other => panic!("{}: p-author is not an h-card: {other:?}", rel(&f, site())),
        }
    }
}

#[test]
fn notes_index_has_h_feed() {
    assert!(
        !parsed(&site().join("notes/index.html"))
            .items_of_type("h-feed")
            .is_empty(),
        "/notes/ has no h-feed"
    );
}

#[test]
fn home_posts_show_an_excerpt() {
    // ADR 2026-10-16-home-post-excerpts: one-word titles need context.
    let home = parsed(&site().join("index.html"));
    let feed = home.items_of_type("h-feed");
    let feed = feed.first().expect("home page has no h-feed");
    assert!(!feed.children.is_empty(), "home h-feed lists no posts");
    for entry in &feed.children {
        assert!(
            entry
                .prop("summary")
                .iter()
                .any(|v| matches!(v, Value::Text(t) if !t.is_empty())),
            "home post {:?} has no p-summary",
            entry.prop("name")
        );
    }
}
