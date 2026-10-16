# Output contract

Invariants every emitted HTML page must satisfy. Each is enforced by a test
in `tests/`.

## Universal (every page)

- Valid HTML5: parses with no errors under the harness's strict checker
  (`tests/harness/support/html.rs`). Why: browsers repair broken markup
  differently, and microformats parsers and feed readers see the raw tree,
  so the only safe markup is markup that needs no repair.
- Passes the W3C Nu HTML Checker with no errors or warnings, including
  heading order (no skipped levels) and one `<h1>` per page. Checked by
  `make validate` in CI's `html` job, not by `tests/` (ADR 2026-10-11-nu-html-checker-in-ci).
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
- Styling follows [design.md](design.md).
- Loading follows [performance.md](performance.md).

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
  - a hidden, empty link to each configured syndication target
  - received webmentions, when given (see [indieweb.md](indieweb.md))
- Show backlinks (incoming wikilinks) when any exist.

## Home (`/`)

- Contains exactly one `h-card` with `p-name` (author's full name),
  `p-nickname` (the short name shown first, see ADR 2026-10-14-site-and-author-naming), `u-url` (site),
  and any `rel=me` links. The author label is the page's `<h1>`.
- Contains an `h-feed` listing recent posts as `h-entry` items.

## List pages (`/notes/`, `/tags/<tag>/`)

- Contain an `h-feed` of `h-entry` items.

## Dates

- Every date the build shows a reader uses the Holocene year, the Gregorian
  year plus 10,000 (`2026-10-11` shows as `12026-10-11`). See
  [ADR 0011](../docs/adr/0011-holocene-years-on-the-page.md).
- Every machine-readable date stays Gregorian: `<time datetime>`, RSS,
  JSON Feed, the sitemap and URLs.
- Enforced by `tests/harness/dates.rs`.

## Error pages

- `/404.html` exists and is valid HTML5.

## Serving under a sub-path

`--base-url` overrides `site.base_url` for one build. When it carries a path
(a GitHub Pages project site such as `https://dijonkitchen.github.io/posse-bliki`),
every root-relative link the pages and feeds emit (`/notes/`, `/style.css`,
alias redirect targets) is prefixed with that path, so the site works there.
With no path, output is unchanged. Enforced by `tests/harness/base_path.rs`.
