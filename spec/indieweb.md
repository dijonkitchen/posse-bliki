# IndieWeb standards supported

Conformance to each item is enforced by a test in `tests/`.

## Built in

| Standard | Spec | Where |
|---|---|---|
| **microformats2 h-entry** | <https://microformats.org/wiki/h-entry> | every post page |
| **microformats2 h-card** | <https://microformats.org/wiki/h-card> | home page |
| **microformats2 h-feed** | <https://microformats.org/wiki/h-feed> | home, `/notes/`, `/tags/<tag>/` |
| **rel=me** | <https://microformats.org/wiki/rel-me> | home page, footer |
| **rel=author** | <https://microformats.org/wiki/rel-author> | post pages |
| **rel=canonical** | every page |
| **u-syndication** | <https://indieweb.org/u-syndication> | post pages with `syndication:` front-matter |
| **RSS 2.0** | every page advertises via `<link rel=alternate>` |
| **JSON Feed 1.1** | every page advertises via `<link rel=alternate>` |
| **Sitemap** | `/sitemap.xml`, advertised in `robots.txt` |

## Wired up via external services (no server needed)

| Standard | Service | Configuration |
|---|---|---|
| **Webmention** (receive) | <https://webmention.io> | `<link rel=webmention>` on every page |
| **Webmention** (display) | webmention.io jf2 export | fetched at deploy, rendered by `--webmentions` |
| **Pingback** (receive) | <https://webmention.io> | `<link rel=pingback>` on every page |
| **Webmention** (send) | <https://brid.gy> (Mastodon, Bluesky) | deploy workflow, for new posts |
| **ActivityPub** | <https://fed.brid.gy> | deploy workflow; follow `@<domain>@web.brid.gy` |
| **WebSub** | <https://websub.rocks/hub> | optional `<link rel=hub>` if enabled |

## Webmentions at deploy time

See [ADR 2026-10-11-webmentions-at-deploy-time](../docs/adr/2026-10-11-webmentions-at-deploy-time.md). The build
does no network I/O; the deploy workflow fetches and sends.

**Receiving.** `bliki --webmentions FILE` reads webmention.io's jf2 export
and, on each post a mention targets, renders a `<section
class="webmentions">` inside the `h-entry`:

- Targets match a post's canonical URL ignoring scheme, a leading `www.`,
  query, fragment and trailing slash.
- `like-of`, `repost-of` and `bookmark-of` become `p-like`, `p-repost` and
  `p-bookmark` `h-cite`s; `in-reply-to` and anything else become `p-comment`
  `h-cite`s with author, date and text.
- Mentions marked `wm-private` are skipped. Content is text only and
  escaped. Only http(s) URLs become links.
- No section is emitted for a post with no mentions, or without the flag.
- An unreadable or invalid file fails the build.

**Sending.** Every post contains `<a href="TARGET" hidden></a>` for each
target in the config's `syndicate_to`, because Bridgy publishes only pages
that link to it. `bliki --out DIR --pending-webmentions FEED` builds
nothing; it compares `DIR/feed.json` with FEED (the feed live before this
deploy) and prints `endpoint<TAB>source<TAB>target` for each new post and
target. It fails, printing nothing, when FEED isn't a JSON Feed or when
more than 5 posts are new. The workflow's `syndicate` job posts those
lines after the deploy, only while the `SEND_WEBMENTIONS` repository
variable is `true`.

Enforced by `tests/harness/webmentions.rs` and the Rust unit tests in
`build/webmention.rs`.

## Out of scope (deliberately)

- **Micropub** server — would require a dynamic backend; not worth it for a
  static site with Obsidian as the editor. If browser/mobile posting becomes
  important, run a Micropub-to-git bridge (e.g. `quill` → GitHub commit).
- **IndieAuth** server — not needed unless we run Micropub. Use
  <https://indielogin.com> as a relying-party-only flow if logging in
  somewhere is required; the home `h-card` + `rel=me` is enough for now.

## Verifying conformance

```sh
cargo test --release --test harness -- microformats indieweb_links webmentions
```

External validators (run manually before major releases):

- <https://indiewebify.me/>
- <https://pin13.net/mf2/>
- <https://validator.w3.org/feed/>
