---
title: How this bliki works
tags:
  - meta
---

A short tour of the conventions used here.

## Authoring

1. Open the `content/` folder as an Obsidian vault.
2. Write notes in plain markdown.
3. Use `[[wiki links]]` to connect ideas. Quartz turns these into real
   links and generates [[backlinks]] automatically.
4. Front-matter `tags:` produce tag pages, e.g. `/tags/meta`.

## Drafts

```yaml
---
title: A half-formed thought
draft: true
---
```

Notes marked `draft: true` are excluded from the build but still render
normally inside Obsidian.

## Aliases & redirects

```yaml
---
title: My canonical title
aliases:
  - older-slug
  - another-name
---
```

Quartz emits HTML redirects from each alias to the canonical URL.

## Publishing

```sh
git add .
git commit -m "new note: ..."
git push
```

GitHub Actions clones Quartz at the pinned version, overlays this
content + config, builds, and deploys to Pages. Usually under a minute.

## Syndicating (POSSE)

The RSS feed at `/index.xml` is the single syndication source. Tools
like [Bridgy Fed](https://fed.brid.gy) or [echofeed](https://echofeed.app)
can repost from RSS to Mastodon, Bluesky, etc., always pointing back
here as canonical.

## Why so few moving parts

Long-term maintainability beats features. Each layer (markdown, git,
GitHub Pages) has been stable for over a decade and has obvious escape
hatches. Quartz is the one component that could change — but because
it's pinned by version and only consumes plain markdown, swapping it
out later (for Hugo, Astro, 11ty…) means a new build script, not a
content rewrite.
