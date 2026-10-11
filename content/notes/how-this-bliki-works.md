---
title: How this bliki works
date: 2026-05-10
tags: [meta]
summary: A short tour of the conventions used in this vault.
---

## Authoring

1. Open the `content/` folder as an Obsidian vault.
2. Write notes in plain markdown. Use `[[wiki links]]` to connect ideas.
3. Tags via front-matter (`tags: [foo]`) or inline (`#foo`) — both work.

## Drafts

```yaml
---
title: A half-formed thought
draft: true
---
```

Drafts are excluded from the build but still render in Obsidian.

## Aliases & redirects

```yaml
---
title: My canonical title
aliases:
  - /old-slug/
  - /another-name/
---
```

Each alias gets a tiny redirect page pointing at the canonical URL.
Once added, never remove an alias — URLs are forever.

## Publishing

```sh
git add . && git commit -m "new note" && git push
```

CI runs the full harness (HTML validity, microformats conformance, link
integrity, feed validity, idempotency) and only deploys if everything
is green.

## Syndicating

Each deploy sends new posts to [Bridgy](https://brid.gy), which reposts
them to Mastodon and Bluesky, and to [Bridgy Fed](https://fed.brid.gy),
which federates them to the fediverse. The copies always point back here
as canonical. Replies, likes and reposts come back as webmentions and show
under the post on the next deploy.

When a syndicated copy goes live, add its URL to the post's front-matter:

```yaml
syndication:
  - https://mastodon.example/@me/12345
  - https://bsky.app/profile/me/post/abc
```

Those URLs render as `u-syndication` links, closing the POSSE loop.
