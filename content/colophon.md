---
title: Colophon
summary: How this site is built and why.
---

This site is built on a deliberately small stack chosen for longevity:

| Layer | Tool |
|---|---|
| Editor | [Obsidian](https://obsidian.md) — local-first, plain markdown |
| Storage | `.md` files in a Git repo |
| Build | ~7k lines of Rust plus generated Unicode and entity tables, standard library only — no crates |
| Toolchain | [Rust](https://www.rust-lang.org) only, pinned in `rust-toolchain.toml`: the build, the test harness and the tools |
| Design | Atkinson Hyperlegible Next and JetBrains Mono, self-hosted; colours checked for AAA contrast in light and dark |
| Hosting | GitHub Pages |
| CI/CD | GitHub Actions |
| Syndication | RSS at `/index.xml`, JSON Feed at `/feed.json` |
| Webmentions | [webmention.io](https://webmention.io) to receive, [Bridgy](https://brid.gy) and [Bridgy Fed](https://fed.brid.gy) to syndicate |

The architecture is **spec + harness + replaceable build**. The build
code is small and can be rewritten in any language; what's permanent is
the contract in `spec/` and the test harness in `tests/`. See
[how-this-bliki-works](notes/how-this-bliki-works.md) for the day-to-day side and the repo's
`docs/adr/` for the reasoning.

Dates here use the [Holocene calendar](https://en.wikipedia.org/wiki/Holocene_calendar):
the year plus 10,000, so 2026 is 12026. It's a small nod to the
[Long Now Foundation](https://longnow.org) and to Kurzgesagt's
Human Era, counting from
roughly when people started building civilisation, about 12,000 years ago.
Feeds and machine-readable dates stay in the usual calendar.

If every tool listed above disappeared tomorrow, the notes would still
open in any text editor. That's the point.
