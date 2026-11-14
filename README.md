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
content/         Your notes: each is published once its `published` box is ticked
spec/            Outcome contracts (what "correct" means)
tests/           Automated harness enforcing the specs
build/           Build implementation — Rust, std only (replaceable)
docs/adr/        Architecture Decision Records (append-only)
AGENTS.md        Operating contract for any agent/human editing this repo
```

## Daily workflow

1. Open the repo as an Obsidian vault, at its root, on laptop and phone
   (ADR 2026-11-14-vault-at-repo-root).
   The Git plugin commits and pushes every change to `main`,
   and every push deploys.
   The vault's `.obsidian/app.json` turns off wikilinks,
   makes new links relative paths,
   puts new notes in `content/notes/`,
   and hides the code, specs and ADRs from search and the graph
   (*Settings → Files and links*); keep it that way.
2. The vault carries the Linter plugin, its code and its settings,
   so there is nothing to install;
   if Obsidian asks, turn on community plugins (*Settings → Community plugins*).
   Five seconds after you stop typing in a note, and when you save it,
   Linter stamps `updated_at` with the current UTC time,
   and gives a note with no `created_at` the time its file was made
   (ADRs 2026-11-12-obsidian-stamps-updated-at and 2026-11-13-created-at).
   It stamps only notes, in `content/`,
   and leaves the README, specs and ADRs alone.
   Don't run its "Lint all files" command, which would stamp every note.
3. Write a new note; it starts in `content/notes/`.
   Until its `published` property is ticked, the build skips it,
   so an unfinished note can't break the deploy
   (ADR 2026-11-14-published-checkbox).
   It's still public on GitHub, like the rest of the repo.
4. To publish, give it the front matter a published note needs
   (a `title` and its dates; see [`spec/content-schema.md`](spec/content-schema.md)),
   then add the `published` property and tick it.
   A published note can't link to an unpublished one: the build fails on that link.
5. Link notes as `[text](other-note.md)` (Obsidian does this for
   you when you type `[[`).

Front matter handles the rest:
`published: true` puts a note on the site (Obsidian shows it as a checkbox),
and the build skips every note without it,
`aliases:` keeps old URLs redirecting (never remove one),
and `syndication:` lists a post's copies elsewhere.
`created_at` is when you made a note,
as a UTC datetime (`date -u +%Y-%m-%dT%H:%M:%SZ` prints one) or a day.
It dates a post, and a future one holds the post back until then.
Obsidian fills it in the first time you edit a new note, and keeps it after;
anywhere else, type it.
`updated_at` says when you last reworked a note, as a UTC datetime (`2026-11-12T04:27:28Z`) or a day,
and moves the note to the top of Home and Notes.
Obsidian stamps it as you edit; anywhere else, type it.
A note with `updated_at` and no `created_at` is an evergreen note:
listed with the rest, but never sent to the feeds or syndicated. [`spec/content-schema.md`](spec/content-schema.md) has
every field.

## Local commands

```sh
make setup       # install the pinned Rust toolchain (once)
make ci          # full CI — lint, test, build (mirrors GitHub Actions)
make test        # run the spec harness and unit tests (cargo test)
make serve       # build, watch content/, serve at http://localhost:8080
make validate    # check built pages with the W3C Nu HTML Checker (needs Java and Node)
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
- Notes index, alias redirects

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
