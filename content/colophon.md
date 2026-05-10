---
title: Colophon
summary: How this site is built and why.
---

This site is built on a deliberately small stack chosen for longevity:

| Layer | Tool |
|---|---|
| Editor | [Obsidian](https://obsidian.md) — local-first, plain markdown |
| Storage | `.md` files in a Git repo |
| Build | ~300 lines of Python (markdown-it-py + Jinja2) |
| Toolchain | [uv](https://docs.astral.sh/uv/) |
| Hosting | GitHub Pages |
| CI/CD | GitHub Actions |
| Syndication | RSS at `/index.xml`, JSON Feed at `/feed.json` |

The architecture is **spec + harness + replaceable build**. The build
code is small and can be rewritten in any language; what's permanent is
the contract in `spec/` and the test harness in `tests/`. See
[[how-this-bliki-works]] for the day-to-day side and the repo's
`docs/adr/` for the reasoning.

If every tool listed above disappeared tomorrow, the notes would still
open in any text editor. That's the point.
