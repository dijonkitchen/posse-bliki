//! Webmentions: rendering the ones received (webmention.io's jf2 export,
//! fetched by the deploy workflow) and listing the ones to send for new
//! posts (Bridgy Publish, Bridgy Fed). No network here: std has no HTTPS,
//! so the workflow does the fetching and posting. See spec/indieweb.md and
//! docs/adr/0008-webmentions-at-deploy-time.md.

use std::collections::HashMap;

use crate::json::{self, Json};
use crate::util::escape as e;

/// One received response to a post.
pub struct Mention {
    kind: Kind,
    url: String,
    author_name: String,
    author_url: Option<String>,
    published: String,
    text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Like,
    Repost,
    Bookmark,
    Reply,
    Mention,
}

/// Only http(s) URLs are ever emitted as links.
fn web_url(u: &str) -> Option<String> {
    let lower = u.to_ascii_lowercase();
    (lower.starts_with("https://") || lower.starts_with("http://")).then(|| u.to_string())
}

/// A URL reduced to host (without `www.`) and path, so a mention of
/// `http://www.example.com/notes/x` matches the post at
/// `https://example.com/notes/x/`.
pub fn url_key(u: &str) -> String {
    let rest = u.split_once("://").map_or(u, |(_, r)| r);
    let rest = rest.split(['#', '?']).next().unwrap_or("");
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    let host = host.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    format!("{}/{}", host, path.trim_end_matches('/'))
}

/// Parse webmention.io's jf2 feed into mentions keyed by `url_key(wm-target)`.
pub fn load(text: &str) -> Result<HashMap<String, Vec<Mention>>, String> {
    let doc = json::parse(text)?;
    let children = doc.get("children").ok_or("webmentions: no \"children\" array")?;
    let mut by_target: HashMap<String, Vec<Mention>> = HashMap::new();
    for c in children.as_arr() {
        if c.get("wm-private") == Some(&Json::Lit("true".into())) {
            continue;
        }
        let (Some(target), Some(url)) = (c.str_at("wm-target"), c.str_at("url").and_then(web_url)) else {
            continue;
        };
        let kind = match c.str_at("wm-property").unwrap_or("mention-of") {
            "like-of" => Kind::Like,
            "repost-of" => Kind::Repost,
            "bookmark-of" => Kind::Bookmark,
            "in-reply-to" => Kind::Reply,
            _ => Kind::Mention,
        };
        let author = c.get("author");
        let author_name = author.and_then(|a| a.str_at("name")).filter(|n| !n.trim().is_empty());
        let author_url = author.and_then(|a| a.str_at("url")).and_then(web_url);
        let author_name = author_name.map(String::from).or_else(|| author_url.clone()).unwrap_or_else(|| "someone".into());
        let published = c.str_at("published").or_else(|| c.str_at("wm-received")).unwrap_or("").to_string();
        let text = c.get("content").and_then(|t| t.str_at("text")).unwrap_or("").trim().to_string();
        by_target.entry(url_key(target)).or_default().push(Mention {
            kind,
            url,
            author_name,
            author_url,
            published,
            text,
        });
    }
    for v in by_target.values_mut() {
        v.sort_by(|a, b| (&a.published, &a.url).cmp(&(&b.published, &b.url)));
        v.dedup_by(|a, b| a.url == b.url && a.kind == b.kind);
    }
    Ok(by_target)
}

fn author(m: &Mention) -> String {
    match &m.author_url {
        Some(u) => format!("<a class=\"p-author h-card\" href=\"{}\">{}</a>", e(u), e(&m.author_name)),
        None => format!("<span class=\"p-author h-card\">{}</span>", e(&m.author_name)),
    }
}

/// The responses section for one post, inside its h-entry. Text only:
/// mention content is never emitted as HTML.
pub fn render(mentions: &[Mention]) -> String {
    if mentions.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n<section class=\"webmentions\">\n<h2>Responses</h2>\n");
    for (kind, prop, label) in [
        (Kind::Like, "p-like", "Liked by"),
        (Kind::Repost, "p-repost", "Reposted by"),
        (Kind::Bookmark, "p-bookmark", "Bookmarked by"),
    ] {
        let people: Vec<String> = mentions
            .iter()
            .filter(|m| m.kind == kind)
            .map(|m| {
                let who = m.author_url.as_deref().unwrap_or(&m.url);
                format!(
                    "<span class=\"{} h-cite\"><a class=\"u-url p-name\" href=\"{}\">{}</a></span>",
                    prop,
                    e(who),
                    e(&m.author_name)
                )
            })
            .collect();
        if !people.is_empty() {
            out.push_str(&format!("<p>{} {}: {}</p>\n", label, people.len(), people.join(", ")));
        }
    }
    let replies: Vec<&Mention> = mentions.iter().filter(|m| matches!(m.kind, Kind::Reply | Kind::Mention)).collect();
    if !replies.is_empty() {
        out.push_str("<ol class=\"replies\">\n");
        for m in replies {
            let date = m.published.get(..10).unwrap_or(&m.published);
            let verb = if m.kind == Kind::Reply { "replied" } else { "mentioned this" };
            let text = if m.text.is_empty() {
                String::new()
            } else {
                let short: String = m.text.chars().take(1000).collect();
                format!("\n<p class=\"p-content\">{}</p>", e(&short))
            };
            out.push_str(&format!(
                "<li class=\"p-comment h-cite\">\n<p>{} {} \u{b7} <a class=\"u-url\" href=\"{}\"><time class=\"dt-published\" datetime=\"{}\">{}</time></a></p>{}\n</li>\n",
                author(m),
                verb,
                e(&m.url),
                e(&m.published),
                e(date),
                text
            ));
        }
        out.push_str("</ol>\n");
    }
    out.push_str("</section>\n");
    out
}

/// More new posts than this in one deploy means the previous feed isn't
/// comparable (a moved domain, a missing feed): send nothing rather than
/// re-syndicate the archive.
pub const MAX_NEW: usize = 5;

/// Webmentions to send: one line per new post and syndication target,
/// `endpoint<TAB>source<TAB>target`. A post is new when its URL is in the
/// `current` feed.json but not the `previous` one.
pub fn pending(previous: &str, current: &str, targets: &[(&str, &str)]) -> Result<Vec<String>, String> {
    let urls = |text: &str, which: &str| -> Result<Vec<String>, String> {
        let feed = json::parse(text).map_err(|m| format!("{} feed: {}", which, m))?;
        let items = feed.get("items").ok_or_else(|| format!("{} feed: no \"items\" array", which))?;
        Ok(items.as_arr().iter().filter_map(|i| i.str_at("url").map(String::from)).collect())
    };
    let seen: Vec<String> = urls(previous, "previous")?.iter().map(|u| url_key(u)).collect();
    let new: Vec<String> = urls(current, "current")?.into_iter().filter(|u| !seen.contains(&url_key(u))).collect();
    if new.len() > MAX_NEW {
        return Err(format!(
            "{} new posts since the previous feed (more than {}); not sending, as the previous feed is probably not comparable",
            new.len(),
            MAX_NEW
        ));
    }
    let mut out = Vec::new();
    for source in new {
        for (target, endpoint) in targets {
            out.push(format!("{}\t{}\t{}", endpoint, source, target));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_ignore_scheme_www_and_trailing_slash() {
        assert_eq!(url_key("https://www.Example.com/notes/x/"), url_key("http://example.com/notes/x#c"));
        assert_ne!(url_key("https://example.com/notes/x/"), url_key("https://example.com/notes/y/"));
    }

    #[test]
    fn pending_lists_only_new_posts() {
        let prev = r#"{"items": [{"url": "https://example.com/notes/a/"}]}"#;
        let cur = r#"{"items": [{"url": "https://example.com/notes/b/"}, {"url": "https://example.com/notes/a/"}]}"#;
        let got = pending(prev, cur, &[("https://brid.gy/publish/mastodon", "https://brid.gy/publish/webmention")]).unwrap();
        assert_eq!(
            got,
            vec!["https://brid.gy/publish/webmention\thttps://example.com/notes/b/\thttps://brid.gy/publish/mastodon"]
        );
    }

    #[test]
    fn pending_refuses_a_flood() {
        let cur: Vec<String> = (0..=MAX_NEW).map(|i| format!("{{\"url\": \"https://example.com/notes/{}/\"}}", i)).collect();
        let cur = format!("{{\"items\": [{}]}}", cur.join(","));
        assert!(pending(r#"{"items": []}"#, &cur, &[("t", "e")]).is_err());
    }

    #[test]
    fn render_escapes_and_drops_unsafe_urls() {
        let src = r#"{"children": [
          {"wm-target": "https://example.com/notes/a/", "wm-property": "in-reply-to", "url": "https://m.example/1",
           "author": {"name": "<b>Eve</b>", "url": "javascript:alert(1)"}, "published": "2026-10-01T00:00:00Z",
           "content": {"text": "<script>x</script>"}},
          {"wm-target": "https://example.com/notes/a/", "wm-property": "like-of", "url": "javascript:x"}
        ]}"#;
        let m = load(src).unwrap();
        let html = render(&m[&url_key("https://example.com/notes/a/")]);
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&lt;b&gt;Eve"));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("Liked by"));
    }
}
