# Output contract

Invariants every emitted HTML page must satisfy. Each is enforced by a test
in `tests/`.

## Universal (every page)

- Valid HTML5 (parses with `html5lib` strict mode, no errors).
- `<!DOCTYPE html>` first, `<html lang="en">`.
- `<meta charset="utf-8">`.
- `<meta name="viewport" content="width=device-width, initial-scale=1">`.
- `<title>` is present and non-empty.
- `<link rel="canonical">` matches the page's canonical URL on `site.base_url`.
- `<link rel="alternate" type="application/rss+xml" href="/index.xml">`.
- `<link rel="alternate" type="application/feed+json" href="/feed.json">`.
- `<link rel="webmention">` and `<link rel="pingback">` (configured target).
- All internal links resolve to a real emitted file.
- No `<script>` tags. (Site is fully functional with JS off.)

## Posts (`content/notes/**`)

In addition to the universal invariants, every post page must:

- Contain exactly one `<article class="h-entry">`.
- That `h-entry` contains:
  - `.p-name` with the title
  - `.dt-published` `<time datetime="…">` from front-matter `date`
  - `.e-content` wrapping the rendered body
  - `.p-author` `h-card` referencing the site author
  - `.u-url` matching the page's canonical URL
  - `.p-summary` if `summary` was set
  - `.p-category` per tag
  - `.u-syndication` per syndication URL
- Show backlinks (incoming wikilinks) when any exist.

## Home (`/`)

- Contains exactly one `h-card` with `p-name` (author), `u-url` (site),
  and any `rel=me` links.
- Contains an `h-feed` listing recent posts as `h-entry` items.

## List pages (`/notes/`, `/tags/<tag>/`)

- Contain an `h-feed` of `h-entry` items.

## Error pages

- `/404.html` exists and is valid HTML5.
