# posse-bliki

A personal [bliki](https://martinfowler.com/bliki/WhatIsaBliki.html) (blog +
wiki) edited in [Obsidian](https://obsidian.md), stored as plain markdown,
built by a small dependency-free Rust program, and deployed to GitHub Pages. Follows
[POSSE](https://indieweb.org/POSSE) and supports
[IndieWeb](https://indieweb.org/) standards (microformats2, h-entry, h-card,
Webmention, RSS, JSON Feed).

The architecture is **spec + harness + replaceable build** — see
[`AGENTS.md`](AGENTS.md) and [`docs/adr/2026-05-10-spec-and-harness-over-implementation`](docs/adr/2026-05-10-spec-and-harness-over-implementation.md).

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
2. Write notes. Link them as `[text](other-note.md)` (Obsidian does this for
   you when you type `[[`) and use `#tags` freely.
3. `git commit && git push`. CI runs the harness and deploys.

## Local commands

```sh
make setup       # install the pinned Rust toolchain (once)
make ci          # full CI — lint, test, build (mirrors GitHub Actions)
make test        # run the spec harness and unit tests (cargo test)
make serve       # build, watch content/, serve at http://localhost:8080
make validate    # check built pages with the W3C Nu HTML Checker (needs Java)
make help        # list all targets
```

Requires [rustup](https://rustup.rs) and `make`, nothing else. rustup reads
`rust-toolchain.toml` and installs the exact Rust version the repo pins, so
every machine and CI run uses the same compiler. Make ships with macOS and
every Linux distro. The build, the test harness and the tools are all Rust
with zero crate dependencies — see
[`docs/adr/2026-10-06-rust-build-zero-dependencies`](docs/adr/2026-10-06-rust-build-zero-dependencies.md) and
[`docs/adr/2026-10-11-rust-only-toolchain`](docs/adr/2026-10-11-rust-only-toolchain.md) for why.

## What gets deployed

- HTML pages with `h-entry` / `h-card` / `h-feed` microformats
- `/index.xml` (RSS 2.0) and `/feed.json` (JSON Feed 1.1); `/feed` and
  `/feed.xml` repeat the RSS for readers subscribed to the old Substack
  and Jekyll feeds
- `/sitemap.xml`, `/robots.txt`, `/404.html`
- Tag pages, notes index, alias redirects

## Hosting

The site is live at <https://www.dijonkitchen.org>
([ADR 2026-10-14-www-as-canonical-host](docs/adr/2026-10-14-www-as-canonical-host.md)),
served by GitHub Pages (Settings → Pages → Source: GitHub Actions). Every
push to `main` runs the harness and deploys. Site settings (title, base
URL, author, rel=me links, webmention.io domain, syndication targets) are
in `default_config()` in [`build/site.rs`](build/site.rs).

## POSSE and webmentions

The deploy workflow handles both directions
([ADR 2026-10-11-webmentions-at-deploy-time](docs/adr/2026-10-11-webmentions-at-deploy-time.md)):

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
