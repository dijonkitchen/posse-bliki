//! Evergreen notes and the order of lists (spec/content-schema.md,
//! ADR 2026-11-07-evergreen-notes): a note with `updated` but no `date` is
//! listed on home and /notes/ but stays out of the feeds, and those lists
//! put the most recently tended note first, by the later of `date` and
//! `updated`. The feeds stay in publish order.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::support::mf2::{self, Item};
use crate::support::{build, html, read, site, temp_dir, vault_copy};

const TODAY: &str = "2099-01-05";
const NOTES: [(&str, &str); 4] = [
    (
        "evergreen-garden",
        "---\ntitle: Evergreen garden\nupdated: 2099-01-03\n---\n\ntended, never posted\n",
    ),
    (
        "tended-post",
        "---\ntitle: Tended post\ndate: 2020-01-01\nupdated: 2099-01-04\n---\n\nold, freshly updated\n",
    ),
    ("new-post", "---\ntitle: New post\ndate: 2099-01-02\n---\n\nnew\n"),
    (
        "evergreen-later",
        "---\ntitle: Evergreen later\nupdated: 2099-02-01\n---\n\nnot out until February\n",
    ),
];

fn built() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let vault = vault_copy("evergreen");
        for (slug, text) in NOTES {
            std::fs::write(vault.join(format!("notes/{slug}.md")), text).unwrap();
        }
        let out = temp_dir("evergreen-out");
        build(&vault, &out, &["--today", TODAY]).expect("build with evergreen notes");
        out
    })
}

fn parsed(path: &Path) -> mf2::Parsed {
    mf2::parse(&html::parse(&read(path)).expect("valid HTML"))
}

/// The entries of the first h-feed on a page, in order.
fn entries(path: &Path) -> Vec<Item> {
    let doc = parsed(path);
    doc.items_of_type("h-feed").first().expect("no h-feed").children.clone()
}

fn url(entry: &Item) -> String {
    entry.texts("url").first().expect("entry without u-url").to_string()
}

fn first_date(entry: &Item, prop: &str) -> Option<String> {
    entry.texts(prop).first().map(|s| s[..10].to_string())
}

#[test]
fn evergreen_note_is_listed_but_not_posted() {
    let out = built();
    assert!(out.join("notes/evergreen-garden/index.html").is_file());
    for page in ["index.html", "notes/index.html", "sitemap.xml"] {
        assert!(
            read(&out.join(page)).contains("/notes/evergreen-garden/"),
            "{page} doesn't list the evergreen note"
        );
    }
    for feed in ["index.xml", "feed.json"] {
        assert!(
            !read(&out.join(feed)).contains("/notes/evergreen-garden/"),
            "{feed} carries the evergreen note, so it would be syndicated"
        );
    }
}

#[test]
fn lists_put_the_most_recently_tended_note_first() {
    let want = ["/notes/tended-post/", "/notes/evergreen-garden/", "/notes/new-post/"];
    for page in ["index.html", "notes/index.html"] {
        let got: Vec<String> = entries(&built().join(page)).iter().take(3).map(url).collect();
        assert_eq!(got, want, "{page} isn't in order of last tended");
    }
}

#[test]
fn feeds_stay_in_publish_order() {
    let json = posse_bliki_build::json::parse(&read(&built().join("feed.json"))).expect("feed.json");
    let items = json.get("items").expect("items").as_arr();
    let urls: Vec<&str> = items.iter().map(|i| i.str_at("url").expect("url")).collect();
    assert!(
        urls[0].ends_with("/notes/new-post/"),
        "newest post isn't first: {}",
        urls[0]
    );
    let dates: Vec<&str> = items.iter().map(|i| i.str_at("date_published").unwrap()).collect();
    assert!(
        dates.windows(2).all(|w| w[0] >= w[1]),
        "feed.json isn't newest first by date_published"
    );
}

#[test]
fn listed_notes_say_when_they_were_updated() {
    let list = entries(&built().join("notes/index.html"));
    let find = |slug: &str| {
        list.iter()
            .find(|e| url(e) == format!("/notes/{slug}/"))
            .unwrap_or_else(|| panic!("{slug} not listed"))
            .clone()
    };
    let tended = find("tended-post");
    assert_eq!(first_date(&tended, "published").as_deref(), Some("2020-01-01"));
    assert_eq!(first_date(&tended, "updated").as_deref(), Some("2099-01-04"));
    let evergreen = find("evergreen-garden");
    assert_eq!(first_date(&evergreen, "published"), None);
    assert_eq!(first_date(&evergreen, "updated").as_deref(), Some("2099-01-03"));
    let new = find("new-post");
    assert_eq!(
        first_date(&new, "updated"),
        None,
        "a never-updated post shows an update"
    );
}

#[test]
fn evergreen_page_dates_its_permalink_by_its_update() {
    let doc = parsed(&built().join("notes/evergreen-garden/index.html"));
    let entry = doc.items_of_type("h-entry");
    let entry = entry.first().expect("no h-entry");
    assert!(
        entry.prop("published").is_empty(),
        "evergreen note claims a publish date"
    );
    assert_eq!(first_date(entry, "updated").as_deref(), Some("2099-01-03"));
    assert!(read(&built().join("notes/evergreen-garden/index.html")).contains(">Updated "));
}

#[test]
fn evergreen_note_waits_for_its_updated_date() {
    let out = built();
    assert!(!out.join("notes/evergreen-later/index.html").exists());
    for page in ["index.html", "notes/index.html", "sitemap.xml"] {
        assert!(
            !read(&out.join(page)).contains("evergreen-later"),
            "{page} lists it early"
        );
    }
}

#[test]
fn note_with_neither_date_nor_updated_fails_the_build() {
    let vault = vault_copy("undated");
    std::fs::write(vault.join("notes/undated.md"), "---\ntitle: Undated\n---\n\nwhen?\n").unwrap();
    let err = build(&vault, &temp_dir("undated-out"), &[]).expect_err("a note needs a date");
    assert!(err.contains("notes/undated.md") && err.contains("evergreen"), "{err}");
}

/// The real site's /notes/ list is in order of last tended.
#[test]
fn notes_list_is_in_order_of_last_tended() {
    let tended: Vec<String> = entries(&site().join("notes/index.html"))
        .iter()
        .map(|e| {
            let p = first_date(e, "published").unwrap_or_default();
            let u = first_date(e, "updated").unwrap_or_default();
            p.max(u)
        })
        .collect();
    assert!(!tended.is_empty(), "/notes/ lists nothing");
    assert!(
        tended.windows(2).all(|w| w[0] >= w[1]),
        "/notes/ isn't most recently tended first: {tended:?}"
    );
}
