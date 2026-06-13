# Feeds

Two feeds are emitted. Both are syndication sources for POSSE — point
external bridges (Bridgy Fed, echofeed, IFTTT) at one of them.

## RSS 2.0 — `/index.xml`

- `<rss version="2.0">` with the `atom` and `content`
  (`http://purl.org/rss/1.0/modules/content/`) namespaces declared.
- Channel: `<title>`, `<link>` (canonical site URL), `<description>`,
  `<language>`, `<lastBuildDate>` (RFC 822), `<atom:link rel="self">`.
- Up to 50 most recent posts (excluding drafts), newest first.
- Each `<item>`: `<title>`, `<link>` (canonical post URL),
  `<guid isPermaLink="true">` = link, `<pubDate>` (RFC 822 from
  front-matter `date`), `<description>` (the `summary` if set, else a
  plain-text excerpt), and `<content:encoded>` (the full HTML-escaped
  rendered body — readers must never need to visit the site to get the
  words; see `portability.md` P6).
- Validates against the W3C feed validator semantics
  (tested via `feedparser`: `bozo == 0`).

## JSON Feed v1.1 — `/feed.json`

Spec: <https://www.jsonfeed.org/version/1.1/>

- Top-level: `version`, `title`, `home_page_url`, `feed_url`,
  `language`, `authors`, `items`.
- Each item: `id` (canonical URL), `url`, `title`, `content_html`,
  `date_published` (RFC 3339), `tags`, optional `summary`.

## Sitemap — `/sitemap.xml`

- `urlset` per <https://www.sitemaps.org/protocol.html>.
- One `<url>` per non-draft page, with `<loc>` and `<lastmod>` (W3C date).

## robots.txt

Tiny:

```
User-agent: *
Allow: /
Sitemap: <site.base_url>/sitemap.xml
```
