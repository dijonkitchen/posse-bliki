# Feeds

Two feeds are emitted. Both are syndication sources for POSSE — point
external bridges (Bridgy Fed, echofeed, IFTTT) at one of them.

## RSS 2.0 — `/index.xml`

- `<rss version="2.0">` with the `atom` and `content` namespaces declared.
- Channel: `<title>`, `<link>` (canonical site URL), `<description>`,
  `<language>`, `<lastBuildDate>` (RFC 822), `<atom:link rel="self">`.
- Up to 50 most recent posts (excluding drafts and evergreen notes),
  newest first by `date`. Why: feed
  readers fetch the whole file on every poll, so it stays small; the JSON
  Feed and the notes index carry everything.
  With full text, 50 posts come to about 100 KB, or 30 KB gzipped.
- Each `<item>`: `<title>`, `<link>` (canonical post URL),
  `<guid isPermaLink="true">` = link, `<pubDate>` (RFC 822 from
  front-matter `date`), `<description>`, then `<content:encoded>`.
- `<description>` is plain text:
  the `summary`, or else the body's opening words with markup stripped,
  at most 200 characters,
  cut at a word with `…` marking the cut.
- `<content:encoded>` is the whole post as HTML, XML-escaped,
  the same HTML the JSON Feed's `content_html` carries.
  Every root-relative `href` and `src` (`/notes/x/`) is made absolute against the site,
  and every fragment link (`#fn1`) against the post,
  because RSS has no way to say what a relative URL is relative to.
  Why: readers get the whole post in their feed reader,
  as they did from Substack and Jekyll,
  and this site keeps the original (ADR 2026-11-07-full-text-rss).
- Description first, then the full text, as the
  [RSS Best Practices Profile](https://www.rssboard.org/rss-profile#namespace-elements-content-encoded)
  recommends for a feed with summaries.
  Tested in `tests/harness/rss.rs`.
- Well-formed XML with the fields above (tested in `tests/harness/rss.rs`
  with the strict XML checker in `tests/harness/support/xml.rs`). Why: feed
  readers reject a feed that isn't well-formed outright, so one bad
  character would silently cut off every subscriber.
- Follows RSS 2.0 as the W3C Feed Validation Service reads it: absolute
  URLs, one `atom:link rel="self"` pointing at the feed, unique permalink
  guids, `isPermaLink` only `true` or `false`, real RFC 822 dates (the
  weekday matches the date), an ISO 639 `<language>` (the validator's own
  code list), no markup in descriptions, and no unknown un-namespaced
  elements.
  In `content:encoded`: strict HTML, absolute links and images,
  and no scripts, frames, plugins, styles or event handlers.
  The service blocks automated requests, so the harness checks
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
- Every post (excluding drafts and evergreen notes), newest first by `date`.
- Each item: `id` (canonical URL), `url`, `title`, `content_html`,
  `date_published` (RFC 3339), optional `summary`.

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

## Follow page — `/follow/`

When the vault has `content/follow.md`, it is an ordinary page that tells
readers the best way to follow the site: subscribe to the RSS feed in a
feed reader, such as FeedFlow, and import the blogroll's OPML to follow
the sites JC reads too (ADR 2026-11-05-follow-with-a-feed-reader). Why:
a feed is the one way to follow that needs no platform, account or
algorithm, and the bare feed link opens as raw XML in a browser, which
tells a newcomer nothing.

- A "Why a feed" section explains it: the feeds and the blogroll are
  open web standards (it links the RSS 2.0, JSON Feed and OPML specs),
  there's no algorithm ranking or rewriting posts, and nothing tracks
  the reader.

- Under the page's text the build lists the full address of `/index.xml`
  and, when there is a blogroll, of `/blogroll.opml`, from the site's
  base URL, to copy into a reader. The build writes them because content
  can't link a site URL (ADR 2026-10-16-relative-markdown-links).
- Every page's footer links `/follow/` first, before the feeds. Without
  `follow.md` there is no footer link.
- `/subscribe/`, the sign-up page when this domain served the Substack,
  redirects to `/follow/` (an alias in `follow.md`).
- Tested in `tests/harness/follow.rs`.

## Sitemap — `/sitemap.xml`

- `urlset` per <https://www.sitemaps.org/protocol.html>.
- One `<url>` per non-draft page, with `<loc>` and `<lastmod>` (W3C date).

## robots.txt

Search engines and every other crawler may read everything. Crawlers
whose operators say what they gather may train AI models are asked to
read nothing (ADR 2026-11-06-no-ai-training). Each is named by the token
its operator documents, in one group, per
[RFC 9309](https://www.rfc-editor.org/rfc/rfc9309):

```
User-agent: *
Allow: /

User-agent: GPTBot
User-agent: ClaudeBot
User-agent: Google-Extended
User-agent: Applebot-Extended
User-agent: meta-externalagent
User-agent: Amazonbot
User-agent: CCBot
Disallow: /

Sitemap: <site.base_url>/sitemap.xml
```

The same companies' search agents (Googlebot, Applebot, OAI-SearchBot,
Claude-SearchBot, Amzn-SearchBot) are never named, so they stay in the
`*` group. Tested in `tests/harness/robots.rs`.
