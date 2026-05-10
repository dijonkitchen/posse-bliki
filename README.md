# posse-bliki

A personal [bliki](https://martinfowler.com/bliki/WhatIsaBliki.html) (blog +
wiki) edited in [Obsidian](https://obsidian.md), stored as plain markdown,
built with [Quartz 4](https://quartz.jzhao.xyz), and deployed to GitHub Pages.

Follows the [POSSE](https://indieweb.org/POSSE) principle: **P**ublish (on
your) **O**wn **S**ite, **S**yndicate **E**lsewhere.

## Repository layout

```
content/              Obsidian vault — your notes (the only thing you edit day-to-day)
quartz.config.ts      Site config (title, baseUrl, theme, plugins)
quartz.layout.ts      Page layout (sidebars, footer, components)
.github/workflows/    GitHub Action that builds & deploys
scripts/bootstrap.sh  Local preview helper
```

The Quartz source itself is **not** vendored in this repo. The build action
(and the local preview script) clone it fresh from a pinned version tag, then
overlay your `content/` and config files. This keeps the repo small and makes
upgrading Quartz a one-line change.

## Daily workflow

1. Open the `content/` folder as a vault in Obsidian.
2. Write notes. Use `[[wiki links]]` to connect them — Quartz turns these
   into real links and auto-generates backlinks.
3. Commit and push to `main`. GitHub Actions builds and deploys.

## Local preview

```sh
./scripts/bootstrap.sh
```

Serves the site at <http://localhost:8080> with live reload.

Requires Node 22+ and git.

## Configuring

- **Site URL & title**: edit `quartz.config.ts` (`baseUrl`, `pageTitle`).
- **Layout & components**: edit `quartz.layout.ts`.
- **Pin a different Quartz version**: change `QUARTZ_VERSION` in
  `.github/workflows/deploy.yml` and in `scripts/bootstrap.sh`.

## Drafts

Add `draft: true` to a note's front-matter to exclude it from the build.
The note still lives in your vault and renders normally in Obsidian.

## Syndicating (POSSE)

The build emits an RSS feed at `/index.xml`. Point a tool like
[Bridgy Fed](https://fed.brid.gy), [echofeed](https://echofeed.app),
or a self-hosted bridge at it to repost to Mastodon, Bluesky, etc.,
keeping this site as the canonical source.

## Why these choices

| Decision | Reason |
|---|---|
| Plain markdown in git | Portable forever; survives any tool change |
| Obsidian as editor | Local-first, no lock-in, just reads the folder |
| Quartz 4 | Native Obsidian wikilinks, backlinks, graph view |
| Quartz fetched at build, not vendored | Small repo, easy version bumps |
| Pinned Quartz version | Reproducible builds; upgrade on your schedule |
| GitHub Pages | Free, stable, no separate hosting account |
