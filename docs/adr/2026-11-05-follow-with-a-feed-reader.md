# 2026-11-05 — Follow with a feed reader

- Date: 2026-11-05
- Status: Accepted
- Decided by: JC, asking to "note best way to follow is to subscribe to
  RSS feed with something like FeedFlow. Suggest this bliki and blog
  roll." Claude chose a follow page linked from every footer, and the
  `/subscribe/` redirect.

## Context

Every page's footer links the RSS feed, the JSON Feed and the blogroll,
but nothing says what to do with them. A feed link opened in a browser
shows raw XML, which tells a reader who has never used a feed reader
nothing. The blogroll's OPML file has the same problem.

Before this site, most of JC's posts were on Substack, which readers
follow by email. A feed reader does the same job without a platform,
an inbox or an account, and keeps working wherever the posts live. It's
also how independent sites read each other, which is what the blogroll
is for.

[FeedFlow](https://www.feedflow.dev/) is a free, open source (Apache
2.0) feed reader for Android, iOS, macOS, Windows and Linux that shows
posts in the order they were published and imports OPML.

## Decision

- `content/follow.md` is an ordinary page at `/follow/`. It says the best
  way to follow is to subscribe to the RSS feed in a feed reader,
  suggests FeedFlow (any reader works), links [About
  Feeds](https://aboutfeeds.com/) for newcomers, and walks through three
  steps: get a reader, add this bliki, import the blogroll.
- Under the text, the build lists the full addresses of `/index.xml` and
  `/blogroll.opml` (the second only when there's a blogroll), built
  from the site's base URL. A reader asks for a whole address, and
  content can't link a site URL (ADR 2026-10-16-relative-markdown-links),
  so the build writes them. A `<wbr>` lets a phone wrap them before the
  file name.
- Every page's footer links **Follow** first, before RSS, JSON Feed,
  Blogroll and Colophon, when the page exists. The raw feed links stay
  for readers who already know what to do with them.
- `/subscribe/` redirects to `/follow/`. When the domain served the
  Substack, its "Subscribe" buttons linked there.
- The blogroll's intro and the About page's "New here?" list link the
  follow page.

## Consequences

- One tap from any page tells a reader how to follow, and the page
  stays one Markdown file JC can edit.
- If JC moves the site, the addresses follow the base URL; nothing in
  the content names the domain.
- The footer is one link longer. It's a wrapping line of links, so on a
  phone it takes one more row in some styles; the header is unchanged.
- The page names one app. If FeedFlow goes away or JC prefers another,
  it's one line of `follow.md` and the harness check that it's named.

## Alternatives considered

- **A line in the footer of every page** ("Follow with a feed reader
  such as FeedFlow"). Always visible, but too little room for the steps
  and the blogroll, and it repeats on every page.
- **A note at the end of every post.** Common on blogs, but it's noise
  in a bliki where notes are linked and reread more than read once.
- **A section on the About or now page.** Readers don't look there for
  a feed, and the footer already holds the feeds, which is where they do
  look (ADR 2026-10-25-blogroll).
- **A header link.** The header holds three links so it stays on one
  row on small phones (ADR 2026-11-05-now-in-the-header).
- **Style the feed with XSLT so it reads as a page in a browser.** Helps
  readers who click the RSS link, but it's more machinery than a page,
  and a newcomer still needs to be told to get a reader.
