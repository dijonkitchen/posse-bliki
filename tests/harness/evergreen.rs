//! Evergreen notes and the order of lists (spec/content-schema.md,
//! ADR 2026-11-07-evergreen-notes): a note with `updated_at` but no `date` is
//! listed on home and /notes/ but stays out of the feeds, and those lists
//! put the most recently tended note first. The feeds stay in publish order.
//!
//! `updated_at` is a UTC datetime or a bare day, and a post only counts as
//! updated once its publish day is over everywhere
//! (ADR 2026-11-12-updated-at-utc-datetimes).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::support::mf2::{self, Item};
use crate::support::{build, html, read, site, temp_dir, vault_copy};

const TODAY: &str = "2099-01-05";
const NOTES: [(&str, &str); 9] = [
    (
        "evergreen-garden",
        "---\ntitle: Evergreen garden\nupdated_at: 2099-01-03\n---\n\ntended, never posted\n",
    ),
    (
        "tended-post",
        "---\ntitle: Tended post\ndate: 2020-01-01\nupdated_at: 2099-01-04T09:30:00Z\n---\n\nold, freshly updated\n",
    ),
    ("new-post", "---\ntitle: New post\ndate: 2099-01-02\n---\n\nnew\n"),
    (
        "evergreen-later",
        "---\ntitle: Evergreen later\nupdated_at: 2099-02-01\n---\n\nnot out until February\n",
    ),
    // The same UTC day as evergreen-garden, earlier and later than its noon.
    (
        "evergreen-morning",
        "---\ntitle: Evergreen morning\nupdated_at: 2099-01-03T08:00:00Z\n---\n\nearly\n",
    ),
    (
        "evergreen-night",
        "---\ntitle: Evergreen night\nupdated_at: 2099-01-03T22:00:00+00:00\n---\n\nlate\n",
    ),
    // Saved at 8:27 PM Pacific on the day it was published: already the
    // next day in UTC, but not an update.
    (
        "evening-post",
        "---\ntitle: Evening post\ndate: 2099-01-01\nupdated_at: 2099-01-02T04:27:28Z\n---\n\nsame evening\n",
    ),
    // Last saved while it was still being written.
    (
        "drafted-ahead",
        "---\ntitle: Drafted ahead\ndate: 2098-12-31\nupdated_at: 2098-12-25T10:00:00Z\n---\n\nscheduled\n",
    ),
    // Saved at the first moment its publish day is over everywhere.
    (
        "next-day-post",
        "---\ntitle: Next day post\ndate: 2098-12-28\nupdated_at: 2098-12-29T12:00:00Z\n---\n\nreworked\n",
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
    let want = [
        "/notes/tended-post/",
        "/notes/evergreen-night/",
        "/notes/evergreen-garden/",
        "/notes/evergreen-morning/",
        "/notes/new-post/",
    ];
    for page in ["index.html", "notes/index.html"] {
        let got: Vec<String> = entries(&built().join(page)).iter().take(5).map(url).collect();
        assert_eq!(got, want, "{page} isn't in order of last tended");
    }
    // Further down than Home shows: a save that isn't an update moves nothing.
    let got: Vec<String> = entries(&built().join("notes/index.html"))
        .iter()
        .skip(5)
        .take(3)
        .map(url)
        .collect();
    assert_eq!(
        got,
        ["/notes/evening-post/", "/notes/drafted-ahead/", "/notes/next-day-post/"]
    );
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

/// The entry of the note at `slug` in the /notes/ list.
fn listed(slug: &str) -> Item {
    entries(&built().join("notes/index.html"))
        .into_iter()
        .find(|e| url(e) == format!("/notes/{slug}/"))
        .unwrap_or_else(|| panic!("{slug} not listed"))
}

/// The h-entry of the note's own page.
fn page_entry(slug: &str) -> Item {
    let doc = parsed(&built().join(format!("notes/{slug}/index.html")));
    let entries = doc.items_of_type("h-entry");
    (*entries.first().expect("no h-entry")).clone()
}

#[test]
fn updated_at_keeps_its_utc_time_for_machines_and_shows_the_day() {
    for entry in [listed("tended-post"), page_entry("tended-post")] {
        assert_eq!(entry.texts("updated"), ["2099-01-04T09:30:00Z"]);
    }
    assert_eq!(page_entry("evergreen-night").texts("updated"), ["2099-01-03T22:00:00Z"]);
    assert_eq!(
        page_entry("evergreen-garden").texts("updated"),
        ["2099-01-03"],
        "a bare day gained a time nobody wrote"
    );
    let page = read(&built().join("notes/tended-post/index.html"));
    assert!(
        page.contains(">updated 12099-01-04</time>"),
        "page doesn't show the UTC day alone"
    );
}

#[test]
fn a_save_on_the_day_a_post_is_published_is_not_an_update() {
    for slug in ["evening-post", "drafted-ahead"] {
        for entry in [listed(slug), page_entry(slug)] {
            assert!(entry.prop("updated").is_empty(), "{slug} says it was updated");
            assert!(!entry.prop("published").is_empty(), "{slug} lost its publish date");
        }
    }
}

#[test]
fn a_post_is_updated_once_its_publish_day_is_over_everywhere() {
    for entry in [listed("next-day-post"), page_entry("next-day-post")] {
        assert_eq!(entry.texts("updated"), ["2098-12-29T12:00:00Z"]);
    }
}

#[test]
fn sitemap_dates_a_page_by_its_later_utc_day() {
    let sitemap = read(&built().join("sitemap.xml"));
    let lastmod = |slug: &str| {
        let after = sitemap
            .split(&format!("/notes/{slug}/</loc>"))
            .nth(1)
            .unwrap_or_else(|| panic!("{slug} not in the sitemap"));
        let start = after.find("<lastmod>").expect("no lastmod") + "<lastmod>".len();
        after[start..start + 10].to_string()
    };
    assert_eq!(lastmod("tended-post"), "2099-01-04");
    assert_eq!(lastmod("evening-post"), "2099-01-02");
    assert_eq!(lastmod("drafted-ahead"), "2098-12-31");
    assert_eq!(lastmod("evergreen-garden"), "2099-01-03");
}

/// Builds the vault with one more note and returns the build's error.
fn build_error(label: &str, front_matter: &str) -> String {
    let vault = vault_copy(label);
    std::fs::write(
        vault.join("notes/probe.md"),
        format!("---\ntitle: Probe\ndate: 2020-01-01\n{front_matter}\n---\n\ntext\n"),
    )
    .unwrap();
    build(&vault, &temp_dir(&format!("{label}-out")), &["--today", TODAY]).expect_err("the build should fail")
}

#[test]
fn the_old_updated_field_says_its_new_name() {
    let err = build_error("old-updated", "updated: 2099-01-04");
    assert!(
        err.contains("notes/probe.md") && err.contains("`updated` is now `updated_at`"),
        "{err}"
    );
}

#[test]
fn updated_at_must_be_a_day_or_a_utc_datetime() {
    for (label, value) in [
        ("offset", "2099-01-04T01:30:00-08:00"),
        ("zoneless", "2099-01-04T09:30:00"),
        ("clock", "2099-01-04T25:00:00Z"),
    ] {
        let err = build_error(&format!("updated-at-{label}"), &format!("updated_at: {value}"));
        assert!(
            err.contains("notes/probe.md") && err.contains("UTC datetime ending in Z") && err.contains(value),
            "{value}: {err}"
        );
    }
}

#[test]
fn evergreen_note_waits_for_its_updated_at_day() {
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
fn note_with_neither_date_nor_updated_at_fails_the_build() {
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
