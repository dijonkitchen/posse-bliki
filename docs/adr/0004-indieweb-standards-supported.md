# 0004 — IndieWeb standards in scope

- Date: 2026-05-10
- Status: Accepted

## Context

POSSE is one practice within a larger IndieWeb stack. Supporting the full
stack (microformats, Webmention, Micropub, IndieAuth, ActivityPub, WebSub)
is feasible for a static site but each layer adds either build complexity
or external-service dependencies.

## Decision

Support the IndieWeb standards that can be enforced by our build and a
test in `tests/`, plus `<link>` advertisements for those that need an
external service. Do **not** run our own dynamic backend.

In scope, enforced by tests:

- microformats2 — `h-entry` on posts, `h-card` on home, `h-feed` on lists
- `rel=me`, `rel=author`, `rel=canonical`
- `u-syndication` for POSSE outlinks
- RSS 2.0, JSON Feed 1.1, sitemap, robots.txt

Wired up via external services, with `<link>` tags in our HTML:

- Webmention — receive via [webmention.io](https://webmention.io)
- Pingback — receive via webmention.io
- ActivityPub — via [Bridgy Fed](https://fed.brid.gy)
- WebSub — optional via [websub.rocks](https://websub.rocks/hub) hub

Out of scope:

- **Micropub server** — would require a dynamic backend. The editor is
  Obsidian; if browser/mobile posting becomes important, integrate a
  Micropub-to-git bridge later (it's purely additive).
- **IndieAuth server** — only meaningful if we run Micropub.

See [`spec/indieweb.md`](../../spec/indieweb.md) for the live list.

## Consequences

- The site is fully IndieWeb-readable: any tool that consumes
  microformats can extract structured posts.
- POSSE is real: every post can declare its `u-syndication` outlinks.
- Adding Micropub later does not require a redesign — it's a new ADR
  + build hook + new tests.

## Alternatives considered

- **Just RSS, no microformats.** What every "blog" does. Rejected —
  microformats2 is the IndieWeb's `<title>`-tag-equivalent; cheap to
  add, foundational for everything else.
- **Self-host Webmention receiver.** Adds a dynamic component. Rejected
  for now; webmention.io has been stable since 2014 and supports an
  export endpoint, so we can migrate the data later.
