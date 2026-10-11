//! RSS 2.0 feed is well-formed and contains the expected items (spec/feeds.md).

use crate::support::html::Element;
use crate::support::{config_str, read, site, xml};

fn channel() -> Element {
    let root = xml::parse(&read(&site().join("index.xml"))).expect("index.xml is well-formed XML");
    assert_eq!(root.name, "rss");
    assert_eq!(root.attr("version"), Some("2.0"));
    root.child("channel").expect("no <channel>").clone()
}

fn text(el: &Element, name: &str) -> String {
    el.child(name).map(|c| c.text()).unwrap_or_default()
}

/// RFC 822 dates as RSS uses them, e.g. `Thu, 04 Jun 2026 00:00:00 +0000`.
fn is_rfc822(s: &str) -> bool {
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let parts: Vec<&str> = s.split(' ').collect();
    let digits = |p: &str, n: usize| p.len() == n && p.bytes().all(|b| b.is_ascii_digit());
    let time = |p: &str| {
        let t: Vec<&str> = p.split(':').collect();
        t.len() == 3 && t.iter().all(|x| digits(x, 2))
    };
    let zone = |p: &str| p == "GMT" || p == "UT" || (p.len() == 5 && p.starts_with(['+', '-']) && digits(&p[1..], 4));
    parts.len() == 6
        && parts[0].strip_suffix(',').is_some_and(|d| days.contains(&d))
        && (digits(parts[1], 1) || digits(parts[1], 2))
        && months.contains(&parts[2])
        && digits(parts[3], 4)
        && time(parts[4])
        && zone(parts[5])
}

#[test]
fn rss_exists() {
    assert!(site().join("index.xml").is_file());
}

#[test]
fn rss_has_channel_metadata() {
    let ch = channel();
    assert_eq!(text(&ch, "title"), config_str("title"));
    assert_eq!(
        text(&ch, "link").trim_end_matches('/'),
        config_str("base_url").trim_end_matches('/')
    );
}

#[test]
fn rss_has_items() {
    let ch = channel();
    let items: Vec<&Element> = ch.elements().filter(|e| e.name == "item").collect();
    assert!(!items.is_empty(), "RSS feed has no items");
    for item in items {
        let title = text(item, "title");
        assert!(!title.is_empty(), "item without a title");
        assert!(!text(item, "link").is_empty(), "{title}: no link");
        assert!(!text(item, "guid").is_empty(), "{title}: no guid");
        let date = text(item, "pubDate");
        assert!(is_rfc822(&date), "{title}: pubDate {date:?} is not RFC 822");
    }
}

#[test]
fn sitemap_is_well_formed() {
    let root = xml::parse(&read(&site().join("sitemap.xml"))).expect("sitemap.xml is well-formed XML");
    assert_eq!(root.name, "urlset");
    assert!(root.elements().all(|u| u.name == "url" && u.child("loc").is_some()));
}
