//! `created_at` takes a UTC datetime as well as a day
//! (spec/content-schema.md, ADR 2026-11-13-created-at): the page shows its
//! UTC day, machines and feeds get the time, a timed post goes live at the
//! first build at or after its moment, and saves in the 24 hours after it
//! aren't updates.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::support::mf2::{self, Item};
use crate::support::{build, html, read, temp_dir, vault_copy};

const NOTES: [(&str, &str); 4] = [
    (
        "timed-post",
        "---\ntitle: Timed post\ncreated_at: 2099-01-02T08:15:00Z\n---\n\nmorning\n",
    ),
    (
        "timed-later",
        "---\ntitle: Timed later\ncreated_at: 2099-01-02T20:00:00+00:00\n---\n\nevening\n",
    ),
    (
        "timed-quick-fix",
        "---\ntitle: Timed quick fix\ncreated_at: 2099-01-02T08:15:00Z\nupdated_at: 2099-01-03T08:14:59Z\n---\n\nsaved within the day\n",
    ),
    (
        "timed-reworked",
        "---\ntitle: Timed reworked\ncreated_at: 2099-01-01T08:15:00Z\nupdated_at: 2099-01-02T08:15:00Z\n---\n\nreworked a day later\n",
    ),
];

fn vault() -> &'static PathBuf {
    static VAULT: OnceLock<PathBuf> = OnceLock::new();
    VAULT.get_or_init(|| {
        let vault = vault_copy("created-at");
        for (slug, text) in NOTES {
            std::fs::write(vault.join(format!("notes/{slug}.md")), text).unwrap();
        }
        vault
    })
}

fn built_at(today: &str) -> PathBuf {
    let out = temp_dir(&format!("created-at-{}", today.replace(':', "")));
    build(vault(), &out, &["--today", today]).expect("build with timed posts");
    out
}

fn built() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| built_at("2099-01-05"))
}

fn page_entry(slug: &str) -> Item {
    let page = read(&built().join(format!("notes/{slug}/index.html")));
    let doc = mf2::parse(&html::parse(&page).expect("valid HTML"));
    let entries = doc.items_of_type("h-entry");
    (*entries.first().expect("no h-entry")).clone()
}

#[test]
fn the_page_shows_the_utc_day_and_machines_get_the_time() {
    assert_eq!(page_entry("timed-post").texts("published"), ["2099-01-02T08:15:00Z"]);
    assert_eq!(
        page_entry("timed-later").texts("published"),
        ["2099-01-02T20:00:00Z"],
        "+00:00 isn't written as Z"
    );
    let page = read(&built().join("notes/timed-post/index.html"));
    assert!(
        page.contains("datetime=\"2099-01-02T08:15:00Z\">12099-01-02</time>"),
        "the page doesn't show the UTC day alone"
    );
}

#[test]
fn feeds_carry_the_time() {
    let rss = read(&built().join("index.xml"));
    assert!(
        rss.contains("<pubDate>Fri, 02 Jan 2099 08:15:00 +0000</pubDate>"),
        "RSS pubDate lacks the time"
    );
    let json = posse_bliki_build::json::parse(&read(&built().join("feed.json"))).expect("feed.json");
    let items = json.get("items").expect("items").as_arr();
    let item = |slug: &str| {
        items
            .iter()
            .position(|i| i.str_at("url").is_some_and(|u| u.ends_with(&format!("/notes/{slug}/"))))
            .unwrap_or_else(|| panic!("{slug} not in feed.json"))
    };
    assert_eq!(
        items[item("timed-post")].str_at("date_published"),
        Some("2099-01-02T08:15:00+00:00")
    );
    assert!(
        item("timed-later") < item("timed-post"),
        "a later time the same day isn't newer"
    );
}

#[test]
fn a_save_within_24_hours_of_created_at_is_not_an_update() {
    let quick = page_entry("timed-quick-fix");
    assert!(quick.prop("updated").is_empty(), "a save 23:59:59 later says updated");
    assert_eq!(
        page_entry("timed-reworked").texts("updated"),
        ["2099-01-02T08:15:00Z"],
        "a save 24 hours later isn't an update"
    );
}

#[test]
fn a_timed_post_goes_live_at_its_moment() {
    let before = built_at("2099-01-02T19:59:59Z");
    assert!(before.join("notes/timed-post/index.html").is_file());
    assert!(
        !before.join("notes/timed-later/index.html").exists(),
        "published before its time"
    );
    assert!(!read(&before.join("feed.json")).contains("timed-later"));
    let at = built_at("2099-01-02T20:00:00Z");
    assert!(at.join("notes/timed-later/index.html").is_file(), "not out at its time");
    let day = built_at("2099-01-02");
    assert!(
        day.join("notes/timed-later/index.html").is_file(),
        "--today with a day doesn't cover the whole day"
    );
}

#[test]
fn created_at_must_be_a_day_or_a_utc_datetime() {
    for (label, value) in [
        ("offset", "2099-01-02T00:15:00-08:00"),
        ("zoneless", "2099-01-02T08:15:00"),
        ("impossible", "2021-02-29"),
    ] {
        let vault = vault_copy(&format!("created-at-bad-{label}"));
        std::fs::write(
            vault.join("notes/probe.md"),
            format!("---\ntitle: Probe\ncreated_at: {value}\n---\n\ntext\n"),
        )
        .unwrap();
        let err =
            build(&vault, &temp_dir(&format!("created-at-bad-{label}-out")), &[]).expect_err("the build should fail");
        assert!(
            err.contains("notes/probe.md")
                && err.contains("`created_at` must be a day")
                && err.contains("UTC datetime ending in Z")
                && err.contains(value),
            "{value}: {err}"
        );
    }
}

#[test]
fn today_takes_a_utc_datetime() {
    let out = temp_dir("created-at-bad-today");
    let err = build(vault(), &out, &["--today", "2099-01-02T12:00:00-08:00"]).expect_err("offset");
    assert!(err.contains("--today"), "{err}");
}
