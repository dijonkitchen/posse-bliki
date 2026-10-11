//! JSON Feed 1.1 (spec/feeds.md).

use posse_bliki_build::json::{self, Json};

use crate::support::{config_str, read, site};

fn feed() -> Json {
    json::parse(&read(&site().join("feed.json"))).expect("feed.json is JSON")
}

#[test]
fn jsonfeed_exists() {
    assert!(site().join("feed.json").is_file());
}

#[test]
fn jsonfeed_top_level() {
    let feed = feed();
    assert_eq!(feed.str_at("version"), Some("https://jsonfeed.org/version/1.1"));
    assert_eq!(feed.str_at("title"), Some(config_str("title")));
    assert_eq!(
        feed.str_at("home_page_url").unwrap().trim_end_matches('/'),
        config_str("base_url").trim_end_matches('/')
    );
    assert!(feed.str_at("feed_url").unwrap().ends_with("/feed.json"));
    assert_eq!(feed.str_at("language"), Some("en"));
    assert!(!feed.get("authors").expect("authors").as_arr().is_empty());
    assert!(!feed.get("items").expect("items").as_arr().is_empty());
}

#[test]
fn jsonfeed_items() {
    for item in feed().get("items").unwrap().as_arr() {
        for key in ["id", "url", "title", "content_html"] {
            assert!(
                item.str_at(key).is_some_and(|v| !v.is_empty()),
                "item without {key}: {:?}",
                item.str_at("id")
            );
        }
        assert!(item.get("date_published").is_some());
    }
}
