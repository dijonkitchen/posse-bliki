# 2026-10-14 — Mobile first, readable output, a Lighthouse budget

- Date: 2026-10-14
- Status: Accepted
- Decided by: JC, asking that the site be "mobile first and nothing is
  mini/uglified. But also scores high on chrome lighthouse/PRPL".

## Context

An audit of the built site with Lighthouse 12 (mobile profile) found:

- The stylesheet was desktop first: the base rules were the wide layout and
  a `max-width: 40rem` query patched it for phones.
- 139 of 145 pages had no `<meta name="description">` (SEO 92).
- No icon, so every page logged a `/favicon.ico` 404 (best practices 96).
- Post images were hot-linked from Substack and xkcd with no size: one was
  a 1.9 MB PNG, giving a 10.9 s Largest Contentful Paint and a layout shift
  (performance 71). Two images on the styling note pointed at `/images/`,
  which the build never emitted.
- A YouTube embed had a fixed 560px width, wider than a phone.
- One post title, "information", is on Lighthouse's list of link texts that
  say nothing out of context (SEO 92 on `/notes/`).

JC also wants the output to stay readable: no minification or uglifying.
The reason is Richard Stallman's: what a site sends should be source
code, the form a person would read and change. Anyone should be able to
audit what a page does and remix it, from "view source" alone, without
hunting for an upstream repository. Minified or uglified code is still
delivered but is no longer source in that sense (Stallman makes the
argument in "The JavaScript Trap").

## Decision

- Write the stylesheet mobile first: phone rules by default, `min-width`
  queries for wider screens, nothing else.
- Ship HTML and CSS exactly as written, so every page can be audited and
  remixed from what the browser receives. The templates already emit one
  element per line and `style.css` is copied as is; tests now hold that.
  Minifying would save only about 3 KB on a page GitHub Pages already
  gzips, so speed gives no reason to give that up.
- Give every indexable page a description, falling back to an excerpt of
  the note's own text.
- Self-host post images in `content/images/` and have the build read each
  one's size from its header to write `width` and `height` (a few dozen
  lines of std-only Rust, `build/image.rs`; no image crate, per ADR 2026-10-06-rust-build-zero-dependencies).
  Lazy-load images after the first, and all embeds.
- Retitle the "information" note to "Information and disinformation", its
  opening line.
- Add a Lighthouse budget as a CI-only job (`make lighthouse`), like the
  HTML checker: it needs Node and Chrome, which the build and `make ci`
  never do.

The full contract is in [spec/performance.md](../../spec/performance.md).

## Consequences

- Every audited page scores 100 in all four Lighthouse categories locally.
- A regression in accessibility, best practices or SEO fails CI outright;
  performance fails below 90.
- Adding an image means putting the file in `content/images/`; a
  hot-linked image fails `tests/harness/performance.rs`.
- The build gains no dependency. CI gains `lighthouse` from npm, run only
  in its own job. It and its 200 transitive packages are locked in
  `scripts/lighthouse/package-lock.json` (registry URLs and integrity
  hashes) and installed with `npm ci --ignore-scripts`, and the job's
  actions are pinned to commit SHAs, per the supply-chain rule in #17.

## Alternatives considered

- **Inline critical CSS** to remove the render-blocking stylesheet: breaks
  the one-stylesheet rule in spec/design.md, repeats the CSS on every page
  and stops it caching between pages, for about 0.1 s on a first visit.
- **A service worker** to pre-cache routes (the P in PRPL): needs
  JavaScript, and the site is JS-free.
- **Resize and convert images in the build** (srcset, WebP): needs an image
  codec, which means a crate or a lot of code. Compressing once when adding
  an image is enough for a handful of images.
- **Lighthouse CI (`@lhci/cli`)**: more moving parts (a config file, an
  upload target) than a pinned `lighthouse` and a small score check.
