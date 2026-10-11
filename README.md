# posse-bliki

A personal [bliki](https://martinfowler.com/bliki/WhatIsaBliki.html) (blog +
wiki) edited in [Obsidian](https://obsidian.md), stored as plain markdown,
built by a small dependency-free Rust program, and deployed to GitHub Pages. Follows
[POSSE](https://indieweb.org/POSSE) and supports
[IndieWeb](https://indieweb.org/) standards (microformats2, h-entry, h-card,
Webmention, RSS, JSON Feed).

The architecture is **spec + harness + replaceable build** — see
[`AGENTS.md`](AGENTS.md) and [`docs/adr/0001`](docs/adr/0001-spec-and-harness-over-implementation.md).

## Layout

```
content/         Obsidian vault — your notes
spec/            Outcome contracts (what "correct" means)
tests/           Automated harness enforcing the specs
build/           Build implementation — Rust, std only (replaceable)
docs/adr/        Architecture Decision Records (append-only)
AGENTS.md        Operating contract for any agent/human editing this repo
```

## Daily workflow

1. Open `content/` as an Obsidian vault.
2. Write notes. Use `[[wiki links]]` and `#tags` freely.
3. `git commit && git push`. CI runs the harness and deploys.

## Local commands

```sh
make ci          # full CI — sync, test, build (mirrors GitHub Actions)
make test        # run the harness only
make serve       # build, watch content/, serve at http://localhost:8080
make validate    # check built pages with the W3C Nu HTML Checker (needs Java)
make help        # list all targets
```

Requires [Rust](https://rustup.rs) (for the build), [uv](https://docs.astral.sh/uv/)
(for the test harness) and `make`. Install uv with
`curl -LsSf https://astral.sh/uv/install.sh | sh`. Make ships with macOS
and every Linux distro. The build itself has zero crate dependencies — see
[`docs/adr/0007`](docs/adr/0007-rust-build-zero-dependencies.md).

## What gets deployed

- HTML pages with `h-entry` / `h-card` / `h-feed` microformats
- `/index.xml` (RSS 2.0) and `/feed.json` (JSON Feed 1.1)
- `/sitemap.xml`, `/robots.txt`, `/404.html`
- Tag pages, notes index, alias redirects

## Going live

1. Check `default_config()` in [`build/site.rs`](build/site.rs)
   (title, base URL, author, rel=me links, webmention.io domain,
   syndication targets).
2. Repo Settings → Pages → Source: GitHub Actions.
3. Push to `main`. CI builds, runs the harness, deploys.

## POSSE and webmentions

The deploy workflow handles both directions
([ADR 0008](docs/adr/0008-webmentions-at-deploy-time.md)):

- **Received** webmentions are fetched from webmention.io and shown on
  each post. Sign in at <https://webmention.io> with the site's domain,
  then save its API key as the `WEBMENTION_IO_TOKEN` repository secret.
  A daily deploy picks up new ones.
- **New posts** are sent to Bridgy Publish (Mastodon, Bluesky) and Bridgy
  Fed (fediverse). Connect your accounts at <https://brid.gy> and
  <https://fed.brid.gy>, then set the `SEND_WEBMENTIONS` repository
  variable to `true`. Until then nothing is sent.

Once a syndicated copy exists, add its URL to the post's `syndication:`
front matter to link to it.
