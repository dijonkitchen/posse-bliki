---
title: Colophon
summary: How this site is built and why.
aliases:
  - /notes/how-this-bliki-works/
---

The stack is small on purpose. Boring tools last longer:

| Layer | Tool |
|---|---|
| Editor | [Obsidian](https://obsidian.md) — local-first, plain markdown |
| Storage | `.md` files in a Git repo |
| Build | ~7k lines of Rust plus generated Unicode and entity tables, standard library only — no crates |
| Toolchain | [Rust](https://www.rust-lang.org) only, pinned in `rust-toolchain.toml`: the build, the test harness and the tools |
| Design | Atkinson Hyperlegible Next and JetBrains Mono, self-hosted; colours checked for AAA contrast in light and dark |
| Hosting | GitHub Pages |
| Domain & DNS | [Porkbun](https://porkbun.com), registrar and nameservers; the zone file is in the repo |
| Email | [Fastmail](https://www.fastmail.com), on the domain |
| CI/CD | GitHub Actions |
| Syndication | RSS at `/index.xml`, JSON Feed at `/feed.json` |
| Blogroll | The sites I read on the [blogroll](blogroll.md), also as OPML for feed readers |
| Webmentions | [webmention.io](https://webmention.io) to receive, [Bridgy](https://brid.gy) and [Bridgy Fed](https://fed.brid.gy) to syndicate |

The build is the replaceable part: **spec + harness + replaceable
build**. It's small enough to rewrite in any language. What sticks around
is the contract in `spec/` and the tests in `tests/` that hold the build
to it. The [repo](https://github.com/dijonkitchen/posse-bliki)'s README has
the day-to-day of writing and publishing, and `docs/adr/` has the whys.

Everything gets published here first, following
[POSSE](https://indieweb.org/POSSE): Publish on your Own Site, Syndicate
Elsewhere. A copy on Mastodon, Bluesky or anywhere else links back here,
and the post lists it under "Also on". Replies to a copy can find their
way back as [webmentions](https://www.w3.org/TR/webmention/) under the
post.

Dates here use the [Holocene calendar](https://en.wikipedia.org/wiki/Holocene_calendar):
the year plus 10,000, so 2026 is 12026. It's a small nod to the
[Long Now Foundation](https://longnow.org) and to Kurzgesagt's
Human Era, counting from
roughly when people started building civilisation, about 12,000 years ago.
Feeds and machine-readable dates stay in the usual calendar.

The Style menu at the top of each page switches between the house style and
a few others, after the CSS Zen Garden; the [style garden](garden.md) is a
page to try them on. Each is one short stylesheet you can read and remix.

If every tool above disappeared tomorrow, the notes would still open in
any text editor. Plain text outlives platforms.
