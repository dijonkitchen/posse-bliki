//! RSS 2.0 feed is well-formed and contains the expected items (spec/feeds.md).

use crate::support::html::Element;
use crate::support::{config_str, iso639, read, site, xml};

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

/// Readers subscribed to the Substack (`/feed`) or Jekyll (`/feed.xml`) feed
/// on this domain keep getting posts.
#[test]
fn old_feed_urls_serve_the_rss() {
    let rss = read(&site().join("index.xml"));
    for old in ["feed", "feed.xml"] {
        assert_eq!(read(&site().join(old)), rss, "/{old} differs from /index.xml");
    }
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

/// Day of the week for a Gregorian date, Monday = 0 (Sakamoto's method),
/// computed here rather than with the build's own date code.
fn weekday(y: i32, m: usize, d: i32) -> usize {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if m < 3 { y - 1 } else { y };
    let sunday_first = (y + y / 4 - y / 100 + y / 400 + T[m - 1] + d).rem_euclid(7);
    ((sunday_first + 6) % 7) as usize
}

/// An RFC 822 date whose weekday matches its calendar date.
fn is_real_rfc822_date(s: &str) -> bool {
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if !is_rfc822(s) {
        return false;
    }
    let p: Vec<&str> = s.split(' ').collect();
    let m = months.iter().position(|x| *x == p[2]).unwrap() + 1;
    let (day, year): (i32, i32) = (p[1].parse().unwrap(), p[3].parse().unwrap());
    let month_len = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m - 1]
        + i32::from(m == 2 && (year % 4 == 0 && year % 100 != 0 || year % 400 == 0));
    (1..=month_len).contains(&day) && days[weekday(year, m, day)] == p[0].trim_end_matches(',')
}

fn is_absolute_http_url(s: &str) -> bool {
    (s.starts_with("https://") || s.starts_with("http://")) && !s.contains(char::is_whitespace)
}

// The checks below follow the RSS 2.0 spec (https://www.rssboard.org/rss-specification)
// and the rules the W3C Feed Validation Service applies. The service itself
// can't be called from CI (see docs/adr/2026-10-11-nu-html-checker-in-ci.md).

#[test]
fn rss_channel_follows_rss_2() {
    let ch = channel();
    for required in ["title", "link", "description"] {
        assert!(
            !text(&ch, required).trim().is_empty(),
            "channel <{required}> is missing or empty"
        );
    }
    assert!(
        is_absolute_http_url(&text(&ch, "link")),
        "channel <link> is not an absolute URL"
    );
    let language = text(&ch, "language");
    assert!(
        iso639::is_valid(&language),
        "channel <language> {language:?} is not an ISO 639 language code"
    );
    let built = text(&ch, "lastBuildDate");
    assert!(
        is_real_rfc822_date(&built),
        "lastBuildDate {built:?} is not a real RFC 822 date"
    );

    let selfs: Vec<&Element> = ch
        .elements()
        .filter(|e| e.name == "atom:link" && e.attr("rel") == Some("self"))
        .collect();
    assert_eq!(selfs.len(), 1, "channel needs exactly one atom:link rel=\"self\"");
    let want = format!("{}/index.xml", config_str("base_url").trim_end_matches('/'));
    assert_eq!(selfs[0].attr("href"), Some(want.as_str()), "atom:link self href");
    assert_eq!(
        selfs[0].attr("type"),
        Some("application/rss+xml"),
        "atom:link self type"
    );

    // Unknown un-namespaced elements are ignored by readers and flagged by validators.
    let known = [
        "title",
        "link",
        "description",
        "language",
        "copyright",
        "managingEditor",
        "webMaster",
        "pubDate",
        "lastBuildDate",
        "category",
        "generator",
        "docs",
        "cloud",
        "ttl",
        "image",
        "rating",
        "textInput",
        "skipHours",
        "skipDays",
        "item",
    ];
    for el in ch.elements() {
        assert!(
            el.name.contains(':') || known.contains(&el.name.as_str()),
            "unknown channel element <{}>",
            el.name
        );
    }
}

#[test]
fn rss_items_follow_rss_2() {
    let ch = channel();
    let known = [
        "title",
        "link",
        "description",
        "author",
        "category",
        "comments",
        "enclosure",
        "guid",
        "pubDate",
        "source",
    ];
    let mut guids = Vec::new();
    for item in ch.elements().filter(|e| e.name == "item") {
        let title = text(item, "title");
        for el in item.elements() {
            assert!(
                el.name.contains(':') || known.contains(&el.name.as_str()),
                "{title}: unknown item element <{}>",
                el.name
            );
            assert!(
                item.elements().filter(|e| e.name == el.name).count() == 1 || el.name == "category",
                "{title}: repeated <{}>",
                el.name
            );
        }
        let link = text(item, "link");
        assert!(
            is_absolute_http_url(&link),
            "{title}: link {link:?} is not an absolute URL"
        );
        let guid = item.child("guid").expect("guid");
        let permalink = guid.attr("isPermaLink");
        assert!(
            matches!(permalink, None | Some("true" | "false")),
            "{title}: isPermaLink must be \"true\" or \"false\", not {permalink:?}"
        );
        if permalink != Some("false") {
            assert!(
                is_absolute_http_url(&guid.text()),
                "{title}: permalink guid is not an absolute URL"
            );
        }
        assert!(!guids.contains(&guid.text()), "{title}: duplicate guid {}", guid.text());
        guids.push(guid.text());
        let date = text(item, "pubDate");
        assert!(
            is_real_rfc822_date(&date),
            "{title}: pubDate {date:?} is not a real date"
        );
        // Descriptions are plain text (spec/feeds.md). No markup at all also rules
        // out what the validator warns about in HTML descriptions: script and other
        // unsafe tags, and relative links or images that readers resolve wrongly.
        let desc = text(item, "description");
        assert!(
            !has_markup(&desc),
            "{title}: description contains HTML markup: {desc:?}"
        );
    }
}

/// `<` that starts a tag, end tag, comment or processing instruction.
fn has_markup(s: &str) -> bool {
    s.as_bytes()
        .windows(2)
        .any(|w| w[0] == b'<' && (w[1].is_ascii_alphabetic() || matches!(w[1], b'/' | b'!' | b'?')))
}

#[test]
fn rss_holds_the_50_newest_posts() {
    let ch = channel();
    let items: Vec<&Element> = ch.elements().filter(|e| e.name == "item").collect();
    assert!(
        items.len() <= 50,
        "RSS feed has {} items; the spec allows 50",
        items.len()
    );
    let feed: Vec<String> = items.iter().map(|i| text(i, "link")).collect();
    let json = posse_bliki_build::json::parse(&read(&site().join("feed.json"))).expect("feed.json");
    let newest: Vec<String> = json
        .get("items")
        .expect("items")
        .as_arr()
        .iter()
        .take(50)
        .map(|i| i.str_at("url").expect("url").to_string())
        .collect();
    assert_eq!(feed, newest, "RSS items are not the 50 newest posts, newest first");
}

#[test]
fn feed_check_helpers() {
    assert!(iso639::is_valid("en") && iso639::is_valid("en-US") && iso639::is_valid("EN"));
    assert!(!iso639::is_valid("english") && !iso639::is_valid("jp") && !iso639::is_valid(""));
    assert!(has_markup("a <script>x</script>") && has_markup("see <a href=\"/x/\">x</a>"));
    assert!(has_markup("<img src=\"pic.png\">") && has_markup("x <!-- y -->"));
    assert!(!has_markup("a < b and c > d") && !has_markup("plain text"));
}

#[test]
fn rfc822_weekday_check() {
    assert!(is_real_rfc822_date("Thu, 04 Jun 2026 00:00:00 +0000"));
    assert!(is_real_rfc822_date("Tue, 29 Feb 2000 00:00:00 +0000"));
    assert!(!is_real_rfc822_date("Fri, 04 Jun 2026 00:00:00 +0000"), "wrong weekday");
    assert!(
        !is_real_rfc822_date("Sat, 29 Feb 2025 00:00:00 +0000"),
        "no 29 Feb in 2025"
    );
    assert!(!is_real_rfc822_date("Thu, 31 Jun 2026 00:00:00 +0000"), "no 31 Jun");
}

#[test]
fn sitemap_is_well_formed() {
    let root = xml::parse(&read(&site().join("sitemap.xml"))).expect("sitemap.xml is well-formed XML");
    assert_eq!(root.name, "urlset");
    assert!(root.elements().all(|u| u.name == "url" && u.child("loc").is_some()));
}
