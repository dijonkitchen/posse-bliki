# 2026-11-07 — Indie web links on the blogroll

- Date: 2026-11-07
- Status: Accepted
- Raised by: JC, about the small-web directories on the listing to-do list:
  "include these links in blogroll for supporting an indie web".

## Context

The blogroll (ADR 2026-10-25-blogroll) lists the sites JC reads,
and the build exports every item on the page to `/blogroll.opml`.
Feed readers import each OPML outline as a subscription
([OPML 2.0](https://opml.org/spec2.opml): "A subscription list is a possibly multiple-level list of subscriptions to feeds").

JC's listing to-dos name the directories, search engines and webrings
that help people find personal sites:
the IndieWeb wiki and webring, Kagi Small Web, Marginalia, Wiby,
ooh.directory, blogroll.org, indieblog.page, personalsit.es, blogs.hn,
nownownow.com, aboutideasnow.com, IndieNews and the size clubs.
Linking them is how a small site helps others get found,
the thing these services exist for.
Marginalia puts it as "Nothing you do to try to make the web a better place
matters if nobody can find what you did"
([About Marginalia Search](https://about.marginalia-search.com/)).

They aren't blogs JC reads.
Most have no feed, and one with a feed would subscribe a reader
to a stream of other people's sites.

## Decision

- `content/blogroll.md` gets a second section,
  "Find more of the indie web", under its own `##` heading,
  one item per service with a line on what it does.
  Google and Bing stay off it; they're on the to-do list for search,
  not for the indie web.
- The OPML exports only the reading list,
  the part of the page before its first `##` heading.
  Sections after it are links on the page only.
- The page's "All of these as one OPML file" line moves from the end of the
  page to right under the reading list, so "these" still means the blogs.

## Consequences

- Importing `/blogroll.opml` still subscribes a reader to the 12 blogs, no more.
- A future category of blogs needs to sit above the first heading,
  or the export rule has to learn about headings.
  That's a change for when JC wants categories.
- `blogroll::split` holds the rule,
  and a harness test checks the section stays out of the OPML
  and the OPML link stays above it.
- The list goes stale like any list of links.
  blogroll.org's submissions, for one, are paused today.

## Alternatives considered

- **Put the services in the same list.** One list, but the OPML would hand
  feed readers directories as if they were blogs.
- **A separate page**, such as `/indieweb/`. Cleaner split,
  but JC asked for the blogroll, where readers already go to find more sites.
- **Nested OPML outlines by heading.** Keeps everything in the file,
  grouped, but still subscribes readers to the feeds of directories.
