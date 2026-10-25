# Feeds

Two feeds are emitted. Both are syndication sources for POSSE — point
external bridges (Bridgy Fed, echofeed, IFTTT) at one of them.

## RSS 2.0 — `/index.xml`

- `<rss version="2.0">` with the `atom` namespace declared.
- Channel: `<title>`, `<link>` (canonical site URL), `<description>`,
  `<language>`, `<lastBuildDate>` (RFC 822), `<atom:link rel="self">`.
- Up to 50 most recent posts (excluding drafts), newest first. Why: feed
  readers fetch the whole file on every poll, so it stays small; the JSON
  Feed and the notes index carry everything.
- Each `<item>`: `<title>`, `<link>` (canonical post URL),
  `<guid isPermaLink="true">` = link, `<pubDate>` (RFC 822 from
  front-matter `date`), `<description>` (plain text: the `summary`, or the
  first 200 characters of the body with markup stripped, XML-escaped).
- Well-formed XML with the fields above (tested in `tests/harness/rss.rs`
  with the strict XML checker in `tests/harness/support/xml.rs`). Why: feed
  readers reject a feed that isn't well-formed outright, so one bad
  character would silently cut off every subscriber.
- Follows RSS 2.0 as the W3C Feed Validation Service reads it: absolute
  URLs, one `atom:link rel="self"` pointing at the feed, unique permalink
  guids, `isPermaLink` only `true` or `false`, real RFC 822 dates (the
  weekday matches the date), an ISO 639 `<language>` (the validator's own
  code list), no markup in descriptions, and no unknown un-namespaced
  elements. The service blocks automated requests, so the harness checks
  these rules itself (ADR 2026-10-11-nu-html-checker-in-ci).

### Old feed URLs — `/feed` and `/feed.xml`

Before this site, the domain served Substack's RSS at `/feed` and the
Jekyll site's at `/feed.xml`. Both carry a byte-for-byte copy of
`/index.xml`, whose `atom:link rel="self"` still names `/index.xml`, so
existing subscribers keep getting posts and readers that honour `self` move
to the canonical URL. Never remove them (ADR 2026-10-16-old-feed-urls).
Tested in `tests/harness/rss.rs`.

## JSON Feed v1.1 — `/feed.json`

Spec: <https://www.jsonfeed.org/version/1.1/>

- Top-level: `version`, `title`, `home_page_url`, `feed_url`,
  `language`, `authors`, `items`.
- Each item: `id` (canonical URL), `url`, `title`, `content_html`,
  `date_published` (RFC 3339), `tags`, optional `summary`.

## Blogroll — `/blogroll/` and `/blogroll.opml`

When the vault has `content/blogroll.md`, it is an ordinary page that
lists the sites JC reads, and the build also emits `/blogroll.opml` from
it (ADR 2026-10-25-blogroll). Why: OPML is the file every feed reader
imports, so a reader can follow the whole list in one step, and the list
stays one Markdown file that reads the same on GitHub and in Obsidian.

- Each list item on the page is one site, written
  `- [Name](https://site/) ([feed](https://site/feed.xml)): why I read it.`
  The first link is the site and must be an absolute web address; a link
  reading `feed` is its feed; the rest of the text is its description.
  An item without a site link is a build error.
- `/blogroll.opml` is OPML 2.0 and well-formed XML: a `<head>` with
  `<title>`, `<ownerName>`, `<ownerId>` (the site) and `<docs>`, and one
  `<outline>` per item in page order: `type="rss"` with `text`, `title`,
  `htmlUrl` and `xmlUrl` when the item has a feed, otherwise `type="link"`
  with `url`; `description` when the item has one. No dates, so the file
  only changes when the list does.
- The blogroll page links the file with
  `<link rel="alternate" type="text/x-opml">` and a visible link under the
  list, and every page's footer links `/blogroll/`. Without
  `blogroll.md` there is no OPML file and no footer link.
- Tested in `tests/harness/blogroll.rs`.

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
