//! Dates people read show Holocene years; machine-readable dates stay ISO
//! (spec/output-contract.md, docs/adr/2026-10-11-holocene-years-on-the-page.md).

use crate::support::regex::Regex;
use crate::support::{html_files, read, rel, site};

/// Every `<time class="dt-…" datetime="…">text</time>` in `html` as `(datetime, text)`.
fn times(html: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    for (i, _) in html.match_indices("<time class=\"dt-") {
        let tag = &html[i..];
        let Some(dt) = tag.find("datetime=\"").map(|s| &tag[s + 10..]) else {
            continue;
        };
        let Some(dt_end) = dt.find('"') else { continue };
        let Some(text) = dt[dt_end..].strip_prefix("\">") else {
            continue;
        };
        let Some(text_end) = text.find("</time>") else { continue };
        out.push((&dt[..dt_end], &text[..text_end]));
    }
    out
}

#[test]
fn visible_years_are_holocene() {
    let mut seen = 0;
    let mut failures = Vec::new();
    for (p, text) in html_files() {
        for (datetime, shown) in times(text) {
            let year: u32 = datetime[..4].parse().expect("datetime starts with a year");
            let want = format!("{}{}", year + 10_000, &datetime[4..10]);
            if shown.strip_prefix("updated ").unwrap_or(shown) != want {
                failures.push(format!("{}: shows {shown:?} for {datetime}", rel(p, site())));
            }
            seen += 1;
        }
    }
    assert!(seen > 0, "no <time> elements found");
    assert!(
        failures.is_empty(),
        "non-Holocene visible dates:\n{}",
        failures.join("\n")
    );
}

#[test]
fn machine_dates_stay_iso() {
    let holocene_iso = Regex::new(r"(^|[^0-9])1\d{4}-\d{2}-\d{2}([^0-9]|$)");
    let holocene_rfc822 = Regex::new(r"(^|[^0-9])\d{2} [A-Z][a-z]{2} 1\d{4}([^0-9]|$)");
    for name in ["index.xml", "feed.json", "sitemap.xml"] {
        let text = read(&site().join(name));
        assert!(
            !holocene_iso.is_match(&text) && !holocene_rfc822.is_match(&text),
            "{name} has a Holocene date"
        );
    }
}
