//! The blogroll: `content/blogroll.md` is an ordinary page, and its list
//! items also become `/blogroll.opml` for feed readers (spec/feeds.md,
//! ADR 2026-10-25-blogroll).

use crate::text;
use crate::util::escape as e;

pub struct Site {
    pub name: String,
    pub url: String,
    pub feed: Option<String>,
    pub description: String,
}

struct Link<'a> {
    href: String,
    text: String,
    /// The whole `<a …>…</a>`.
    html: &'a str,
}

fn links(html: &str) -> Vec<Link<'_>> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(p) = html[at..].find("<a href=\"") {
        let start = at + p;
        let href_start = start + "<a href=\"".len();
        let Some(q) = html[href_start..].find('"') else { break };
        let Some(close) = html[href_start..].find("</a>") else {
            break;
        };
        let end = href_start + close + "</a>".len();
        let inner = &html[href_start..href_start + close];
        let text = inner.find('>').map_or("", |g| &inner[g + 1..]);
        out.push(Link {
            href: text::unescape(&html[href_start..href_start + q]),
            text: text::unescape(&text::strip_html(text)),
            html: &html[start..end],
        });
        at = end;
    }
    out
}

fn is_web(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

/// `text` read on its own: it follows a name on the page, so it may start lower case.
fn sentence(text: &str) -> String {
    let mut c = text.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

/// The sites listed on the rendered blogroll page. In each list item the
/// first link is the site, a link reading "feed" is its feed, and the
/// rest of the text describes it.
pub fn sites(html: &str) -> Result<Vec<Site>, String> {
    let mut out = Vec::new();
    for (i, item) in html.split("<li>").skip(1).enumerate() {
        let item = &item[..item.find("</li>").unwrap_or(item.len())];
        let found = links(item);
        let site = found
            .first()
            .filter(|l| is_web(&l.href))
            .ok_or_else(|| format!("blogroll.md: item {} doesn't start with a link to a site", i + 1))?;
        let feed = found.iter().skip(1).find(|l| l.text.eq_ignore_ascii_case("feed"));
        if let Some(f) = feed.filter(|f| !is_web(&f.href)) {
            return Err(format!(
                "blogroll.md: item {}'s feed {} isn't a web address",
                i + 1,
                f.href
            ));
        }
        let mut rest = item.replacen(site.html, "", 1);
        if let Some(f) = feed {
            rest = rest.replacen(&format!("({})", f.html), "", 1).replacen(f.html, "", 1);
        }
        let rest = text::unescape(&text::strip_html(&rest));
        let description = rest.split_whitespace().collect::<Vec<_>>().join(" ");
        out.push(Site {
            name: site.text.clone(),
            url: site.href.clone(),
            feed: feed.map(|f| f.href.clone()),
            description: sentence(description.trim_start_matches([':', '\u{2014}', '\u{2013}', '-', ' '])),
        });
    }
    Ok(out)
}

/// OPML 2.0 (<https://opml.org/spec2.opml>), the format feed readers
/// import subscriptions from. A site without a feed is a `link` outline.
pub fn opml(title: &str, owner: &str, owner_url: &str, sites: &[Site]) -> String {
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<opml version=\"2.0\">\n<head>\n<title>{}</title>\n\
<ownerName>{}</ownerName>\n<ownerId>{}</ownerId>\n<docs>https://opml.org/spec2.opml</docs>\n</head>\n<body>",
        e(title),
        e(owner),
        e(owner_url)
    );
    for s in sites {
        let description = if s.description.is_empty() {
            String::new()
        } else {
            format!(" description=\"{}\"", e(&s.description))
        };
        match &s.feed {
            Some(feed) => out.push_str(&format!(
                "\n<outline type=\"rss\" text=\"{n}\" title=\"{n}\" htmlUrl=\"{}\" xmlUrl=\"{}\"{description}/>",
                e(&s.url),
                e(feed),
                n = e(&s.name),
            )),
            None => out.push_str(&format!(
                "\n<outline type=\"link\" text=\"{}\" url=\"{}\"{description}/>",
                e(&s.name),
                e(&s.url),
            )),
        }
    }
    out.push_str("\n</body>\n</opml>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_site_feed_and_description() {
        let html = "<ul>\n<li><a href=\"https://a.example/\">A &amp; B</a> (<a href=\"https://a.example/feed.xml\">feed</a>): writes about <em>things</em>.</li>\n\
<li><a href=\"https://c.example/\">C</a></li>\n</ul>";
        let s = sites(html).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].name, "A & B");
        assert_eq!(s[0].url, "https://a.example/");
        assert_eq!(s[0].feed.as_deref(), Some("https://a.example/feed.xml"));
        assert_eq!(s[0].description, "Writes about things.");
        assert_eq!(s[1].feed, None);
        assert_eq!(s[1].description, "");
    }

    #[test]
    fn an_item_without_a_site_link_is_an_error() {
        assert!(sites("<ul><li>just text</li></ul>").is_err());
        assert!(sites("<ul><li><a href=\"/notes/\">local</a></li></ul>").is_err());
    }

    #[test]
    fn opml_escapes_and_types_outlines() {
        let s = sites("<li><a href=\"https://a.example/?x=1&amp;y=2\">A \"quoted\"</a> (<a href=\"https://a.example/f\">feed</a>)</li><li><a href=\"https://b.example/\">B</a></li>").unwrap();
        let x = opml("T", "O", "https://o.example", &s);
        assert!(x.contains("type=\"rss\" text=\"A &#34;quoted&#34;\""));
        assert!(x.contains("htmlUrl=\"https://a.example/?x=1&amp;y=2\""));
        assert!(x.contains("<outline type=\"link\" text=\"B\" url=\"https://b.example/\"/>"));
    }
}
