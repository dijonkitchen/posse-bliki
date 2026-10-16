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
