# posse-bliki

A personal [bliki](https://martinfowler.com/bliki/WhatIsaBliki.html) (blog +
wiki) edited in [Obsidian](https://obsidian.md), stored as plain markdown,
built by a small Python script, and deployed to GitHub Pages. Follows
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
build/           Build implementation (replaceable)
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
make help        # list all targets
```

Requires [uv](https://docs.astral.sh/uv/) and `make`. Install uv with
`curl -LsSf https://astral.sh/uv/install.sh | sh`. Make ships with macOS
and every Linux distro.

## What gets deployed

- HTML pages with `h-entry` / `h-card` / `h-feed` microformats
- `/index.xml` (RSS 2.0) and `/feed.json` (JSON Feed 1.1)
- `/sitemap.xml`, `/robots.txt`, `/404.html`
- Tag pages, notes index, alias redirects

## Going live

1. Edit `site` config at the top of [`build/build.py`](build/build.py)
   (title, base URL, author, rel=me links, webmention.io username).
2. Repo Settings → Pages → Source: GitHub Actions.
3. Push to `main`. CI builds, runs the harness, deploys.

## POSSE

The RSS/JSON feeds are the syndication source. Point an external bridge
([Bridgy Fed](https://fed.brid.gy), [echofeed](https://echofeed.app),
or similar) at `/index.xml` to repost to Mastodon/Bluesky/etc. Each
post can declare its `u-syndication` outlinks via front-matter once
syndicated, closing the POSSE loop.
