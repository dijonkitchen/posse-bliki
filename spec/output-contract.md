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
- `<meta http-equiv="Content-Security-Policy">` right after the charset,
  allowing only the site's own scripts, styles, images, fonts, manifest and
  fetches (`default-src 'none'`, `base-uri 'none'`, `form-action 'none'`),
  plus the page's one inline `<script>` and one `<style>`, each by the
  `'sha256-…'` hash of its text, which starts and ends with no whitespace
  so the Nu HTML Checker can check the hash too
  (ADR 2026-10-28-csp-meta-tag, ADR 2026-11-11-inline-house-style-and-script). Pages load nothing the
  policy would block: no other inline scripts or styles, no `style` or
  event handler attributes, embeds, or files from other hosts.
- `<title>` is present and non-empty.
- `<link rel="canonical">` matches the page's canonical URL on `site.base_url`.
- `<link rel="alternate" type="application/rss+xml" href="/index.xml">`.
- `<link rel="alternate" type="application/feed+json" href="/feed.json">`.
- `<link rel="webmention">` (configured target). No `<link rel="pingback">`:
  Webmention replaces it (ADR 2026-11-06-drop-pingback).
- All internal links resolve to a real emitted file.
- No `<script>` except `<script data-root="/" data-styles="…">`, last in
  `<head>`, holding `build/static/style-picker.js` byte for byte but for
  its final newline.
  `data-styles` lists the picker's styles
  (ADR 2026-10-30-style-before-first-paint), and `data-root` is the site
  root it resolves `themes/` and `sw.js` against
  (ADR 2026-11-11-inline-house-style-and-script). The site is fully
  functional with JS off; the script only loads and remembers the picked
  style, and starts `/sw.js` for offline reading
  (ADR 2026-10-18-style-picker, ADR 2026-10-20-zen-garden-styles,
  ADR 2026-10-29-offline-reading).
- Styling follows [design.md](design.md).
- Loading follows [performance.md](performance.md).

## Posts (`content/notes/**`)

In addition to the universal invariants, every post page must:

- Contain exactly one `<article class="h-entry">`.
- That `h-entry` contains:
  - `.p-name` with the title
  - `.dt-published` `<time datetime="…">` from front-matter `date`;
    an evergreen note has `.dt-updated` from `updated` instead
    (ADR 2026-11-07-evergreen-notes)
  - `.e-content` wrapping the rendered body
  - `.p-author` `h-card` referencing the site author
  - `.u-url` matching the page's canonical URL
  - `.p-summary` if `summary` was set
  - `.u-syndication` per syndication URL
  - a hidden, empty link to each configured syndication target
  - received webmentions, when given (see [indieweb.md](indieweb.md))
- Show backlinks (incoming links from other notes) when any exist.

## Home (`/`)

- Contains exactly one `h-card` with `p-name` (the nickname and full name,
  "JC (Jonathan Chen)", ADR 2026-11-06-profile-name),
  `p-nickname` (the short name shown first, see ADR 2026-10-14-site-and-author-naming), `u-url` (site),
  shown as a "By JC" byline in the footer, not a heading; the name is a
  `<data>` value.
  `p-note` is a one-line bio, also a `<data>` value
  (ADR 2026-11-06-h-card-bio-note).
  `u-photo` is the absolute URL of a square photo in `content/images/`,
  also a `<data>` value, so browsers don't download it
  (ADR 2026-11-06-h-card-photo). `rel=me` profile links are `<link>`s in the `<head>`
  (ADR 2026-10-16-home-shows-recent-posts).
- The site title is the page's `<h1>`, in the header (ADR 2026-10-16-site-title-heads-home).
- Contains an `h-feed` headed "Recent notes" listing the 10 most recently
  tended notes as `h-entry` items, each with a
  `p-summary` excerpt (its `summary`, else the start of its text; ADR
  2026-10-16-home-post-excerpts).
  Order and dates follow [content-schema.md](content-schema.md#evergreen-notes-and-the-order-of-lists).

## List pages (`/notes/`)

- Contain an `h-feed` of `h-entry` items: every post and evergreen note,
  most recently tended first
  (see [content-schema.md](content-schema.md#evergreen-notes-and-the-order-of-lists)).

## Dates

- Every date the build shows a reader uses the Holocene year, the Gregorian
  year plus 10,000 (`2026-10-11` shows as `12026-10-11`). See
  [ADR 2026-10-11-holocene-years-on-the-page](../docs/adr/2026-10-11-holocene-years-on-the-page.md).
- Every machine-readable date stays Gregorian: `<time datetime>`, RSS,
  JSON Feed, the sitemap and URLs.
- Enforced by `tests/harness/dates.rs`.

## Error pages

- `/404.html` exists and is valid HTML5. GitHub Pages serves it, with a 404
  status, for every address that has no page, at any depth, so every link
  and asset in it is root-relative or absolute, never relative. It's
  `noindex`, has no canonical and is left out of the sitemap (Pages also
  serves it at `/404.html` itself, with a 200). Its text links the home
  page, `/notes/` and, when the vault has one, `/follow/`. The service
  worker never keeps it: it stores 200 responses only, so offline a
  reader gets the offline page, not a kept 404, for an address that may
  since have been published. Enforced by `tests/harness/not_found.rs`
  (ADR 2026-11-06-not-found-page).
- `/offline.html` exists, is valid HTML5 and `noindex`, and is left out of
  the sitemap. The service worker shows it for a page that wasn't kept
  when the reader is offline. Like the 404, it has no canonical.
  Enforced by `tests/harness/offline.rs` (ADR 2026-10-29-offline-reading).

## Serving under a sub-path

`--base-url` overrides `site.base_url` for one build. When it carries a path
(a GitHub Pages project site such as `https://dijonkitchen.github.io/posse-bliki`),
every root-relative link the pages and feeds emit (`/notes/`, `/style.css`,
alias redirect targets, the style picker's `data-root`) is prefixed with
that path, and so are the house style's font URLs before the CSP hashes
are taken, so the site works there.
With no path, output is unchanged. Enforced by `tests/harness/base_path.rs`.
