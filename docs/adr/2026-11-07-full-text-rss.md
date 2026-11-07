# 2026-11-07 — Full posts in the RSS feed

- Date: 2026-11-07
- Status: Accepted
- Decided by: JC ("go for it"),
  after Claude listed the RSS snippet as an assumption from the first scaffold
  and recommended full text in `content:encoded` with a summary in `description`.

## Context

The RSS feed came in with the first scaffold
([PR #1](https://github.com/dijonkitchen/posse-bliki/pull/1)),
and no decision chose what its items carry.
Each item had only a `<description>`:
the post's `summary`, or else the first 200 characters of its text,
cut wherever the 200th character fell, often mid-word ("AI may kn").
A reader saw a teaser and had to click through for the rest.

The other feeds don't work that way:

- The JSON Feed already carries every post in full, in `content_html`.
- The Substack feed that readers subscribed to at `/feed`
  carries each post in full in `content:encoded`, next to a short `description`.
- The Jekyll feed at `/feed.xml` put the whole post in `description`.

Both old URLs now serve this site's RSS (ADR 2026-10-16-old-feed-urls),
so readers who followed the old feeds lost the full text without choosing to.

RSS 2.0 defines `description` as "the item synopsis"
([RSS 2.0 specification](https://www.rssboard.org/rss-specification#hrelementsOfLtitemgt)).
The [content module](https://web.resource.org/rss/1.0/modules/content/)
adds `content:encoded` for an item's full content as HTML.
The [RSS Best Practices Profile](https://www.rssboard.org/rss-profile#namespace-elements-content-encoded)
says a publisher with summaries should put the summary in `description`
and the full content in `content:encoded`, "ordering description first within the item".
Its tests found that "all of the tested aggregators support the use of content:encoded",
and that when an item has both,
"six of the eight aggregators display content:encoded and ignore description".

The same profile says a description "should not contain relative URLs,
because the RSS format does not provide a means to identify the base URL of a document"
([§ description](https://www.rssboard.org/rss-profile#element-channel-item-description)).
Posts link other notes as `/notes/x/` and show images from `/images/`,
and footnotes link `#fn1`, which a reader can't resolve on its own.

## Decision

Each RSS item carries the whole post:

- `<content:encoded>` holds the post's HTML, the same HTML as the JSON Feed's `content_html`,
  with root-relative `href` and `src` values made absolute against the site,
  and fragment links made absolute against the post.
  `<rss>` declares the content namespace.
- `<description>` stays plain text and comes first:
  the `summary`, or else the post's opening words, at most 200 characters,
  now cut at a word, with `…` marking the cut,
  the same way the build already writes `<meta name="description">`.
- The feed keeps its 50 newest posts.

The harness checks that every item's `content:encoded` holds the same words as the post,
comes after `description`, parses as strict HTML,
has only absolute links and images,
and has nothing a reader would strip as unsafe,
which are the W3C Feed Validation Service's warnings for feed HTML.
It also checks that a description without a summary ends at a word.

## Consequences

- Readers read whole posts in their feed reader,
  as they did from Substack and Jekyll.
  The canonical copy and the replies still live on the site,
  and every item links to it.
- `/index.xml` grows from about 22 KB to about 100 KB, or from 6 KB to 30 KB gzipped.
  Feed readers fetch it on every poll,
  but GitHub Pages answers conditional requests with `304 Not Modified`
  when nothing changed, so most polls cost little.
- Readers that already hold an item may show it again with the full text,
  because some treat changed content as an update.
- Tools that read the feed get the whole post too.
  That includes Bridgy Fed, which reads a site's feed
  until the site sends it a webmention,
  and "includes your posts' full content when possible"
  ([Bridgy Fed docs](https://fed.brid.gy/docs)).
  `SEND_WEBMENTIONS` is still off, so fediverse followers would see whole posts as well.
  Bridgy Publish reads the post's page, so it is unaffected.
- A new kind of URL in post HTML, such as `srcset`,
  fails the harness until the build makes it absolute too.
- The JSON Feed still carries root-relative URLs in `content_html`.
  JSON Feed 1.1 says nothing about resolving them,
  so this decision leaves it alone.

## Alternatives considered

- **Put the whole post in `description`.**
  The profile recommends this only for publishers without summaries, where it has the widest support.
  This site has summaries,
  and keeping `description` plain keeps the stricter no-markup rule from ADR 2026-10-11-nu-html-checker-in-ci.
- **Keep the snippet and only fix the cut.**
  Readers would still have to click through for every post,
  while the JSON Feed already hands the full text to anyone who asks.
- **Fewer items, to keep the file small.**
  30 KB gzipped for 50 posts is small already,
  and readers who subscribe late would see less of the archive.
- **Switch to Atom, which has `summary` and `content`.**
  Every subscriber is on an RSS URL,
  and RSS with `content:encoded` does the same job.
