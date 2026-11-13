# 2026-11-11 — The house style and the style picker ship inline

- Date: 2026-11-11
- Status: Accepted
- Decided by: JC, after a Lighthouse report on www.dijonkitchen.org
  flagged `/style.css` and `/style-picker.js` as render-blocking.
  JC chose "Inline style and script" over inlining the script only or
  keeping both files.

## Context

Lighthouse's
[render-blocking requests](https://developer.chrome.com/docs/performance/insights/render-blocking)
insight listed `/style-picker.js` (2.6 KiB, 450 ms) and `/style.css`
(5.9 KiB, 150 ms).
Its [network dependency tree](https://developer.chrome.com/docs/performance/insights/network-dependency-tree)
showed both one step behind the page, on a 293 ms critical path.
Its [cache insight](https://developer.chrome.com/docs/performance/insights/cache)
listed both, and the body font, at the 10-minute lifetime GitHub Pages
sends with everything (`Cache-Control: max-age=600`).

Both files block rendering on purpose.
The house style has to be there for the first paint.
The style picker has to set `<html data-style>` and add the picked
style's sheet before it, or the page shifts as the style arrives
(ADR 2026-10-30-style-before-first-paint).
Deferring either one brings that back.
The only way off the critical path is to arrive with the page,
which is what Chrome's guidance says to do:
"Inline requests that are critical for first-paint, but small."

The two files load in parallel,
so the first paint waits one round trip after the page arrives.
Inlining only one of them leaves that round trip in place.
A saved or seasonal style waits longer:
page, then script, then the style's sheet.

Measured on a local copy served like Pages (HTTP/2, gzip,
`max-age=600`), in Chromium with Lighthouse's phone profile and the
network throttled to 150 ms latency and 1.6 Mbps.
These are medians of 7 cold loads:

| Build | Home | Note with a photo | Saved Diablo |
|---|---|---|---|
| Both files | 512 ms | 508 ms | 620 ms |
| Script inline | 492 ms | 512 ms | 580 ms |
| Both inline | 324 ms | 332 ms | 508 ms |

Lighthouse's own simulated first paint barely moved (828 ms before,
859 ms after, home page).
Its model charges the larger page more than the round trip it saves.
It scores 100 either way.

Gzipped, home grows from 3.1 KB to 10.3 KB and a long note from 4.7 KB
to 11.9 KB.
A server's first flight of data is 10 TCP segments, about 14 KB
([RFC 6928](https://www.rfc-editor.org/rfc/rfc6928)),
so a page and everything its first paint needs still arrive together.

## Decision

- `build/static/style.css` ships inside a `<style>` in every page's
  `<head>`, byte for byte but for its final newline, in place of
  `<link rel="stylesheet" href="/style.css">`.
  Its font URLs start at the site root (`url("/fonts/…")`),
  because inline they resolve against each page.
  Under a sub-path the build adds the prefix before taking the hash.
- `build/static/style-picker.js` ships inside
  `<script data-root="/" data-styles="…">`, still last in `<head>`.
  An inline script has no URL of its own, so it resolves `themes/` and
  `sw.js` against `data-root`.
  The build prefixes that like any other root-relative link.
- The Content Security Policy keeps `'self'` and adds a hash for each
  block: `script-src 'self' 'sha256-…'; style-src 'self' 'sha256-…'`.
  A hash allows exactly that text and nothing else
  ([CSP Level 3, hash-source](https://www.w3.org/TR/CSP3/#grammardef-hash-source)),
  so any other inline script or style is still refused, and
  `'unsafe-inline'` stays out.
  `script-src 'self'` stays because
  [`worker-src`](https://www.w3.org/TR/CSP3/#directive-worker-src),
  which covers `/sw.js`, falls back to it.
- Neither block starts or ends with whitespace.
  Browsers hash the text exactly as it is.
  The Nu HTML Checker (ADR 2026-10-11-nu-html-checker-in-ci) trims it
  first, so a block that began with a newline failed its check while
  browsers ran it.
  With nothing to trim, both agree, and `make validate` checks every
  page's hashes independently of the build.
- The build computes both hashes with SHA-256 written against the
  Rust standard library (`build/sha256.rs`; no crates,
  ADR 2026-10-06-rust-build-zero-dependencies).
  Its tests use the examples from
  [FIPS 180-4](https://csrc.nist.gov/pubs/fips/180-4/upd1/final),
  [RFC 4648](https://www.rfc-editor.org/rfc/rfc4648)'s base64 vectors,
  and the CSP spec's own example hash.
  Editing the CSS or the script never needs a manual update.
- `/style.css` is still emitted, for anyone who wants to read or copy
  the house style on its own, and for pages cached before this change.
  `/style-picker.js` is not emitted any more.
  An old cached page that asks for it gets a 404 and works as with
  JavaScript off.
- The offline worker no longer keeps either file on install,
  since every page carries both.
- `tests/harness/csp.rs` checks that every page's hashes match its own
  inline blocks, `design.rs` checks that both blocks ship byte for byte,
  and `base_path.rs` checks the same under a sub-path.

## Consequences

- A first visit, or any visit after the 10-minute cache lifetime,
  paints one round trip sooner.
  A saved or seasonal style saves a round trip too:
  its sheet starts loading while the page is still arriving.
- No style flash and no layout shift: with `diablo.css` delayed 400 ms,
  a saved Diablo still shifts the page 0.
  With JavaScript off the house style is inline as before,
  and the picker works through `:has()` and the `<noscript>` sheets.
- Lighthouse's render-blocking and dependency-tree findings clear.
  Its cache finding is down to the body font.
- Every page is about 7 KB heavier gzipped, on every view.
  A reader's second page within 10 minutes used to reuse the cached
  files.
  Now it carries them again.
  After 10 minutes the old way paid a round trip to re-check both files.
- A single page weighs the same, about 65 KB uncompressed for home, since
  the bytes moved into the HTML rather than being added.
  The size clubs that weigh one page load are unaffected.
- View-source opens with the house style (about 540 lines) and the
  script (about 120) before the page's own markup.
  Both stay unminified (ADR 2026-10-14-mobile-first-readable-output-lighthouse).
- The 10-minute cache lifetime is left alone.
  Pages can't change it.
  The font row doesn't hold up the first paint:
  text shows in the size-matched fallback until the font arrives
  (ADR 2026-10-20-metric-matched-fallback-fonts).
  Once the 10 minutes pass, a re-check returns `304 Not Modified`,
  not the font again
  ([RFC 9111](https://www.rfc-editor.org/rfc/rfc9111)).

## Alternatives considered

- **Inline the script only.** About 2 KB a page.
  But `style.css` still blocks, so the round trip stays
  (492 ms against 512 ms), and the CSS finding stays.
- **Keep both files.** Smallest pages and the shortest view-source,
  but the findings stay and so does the round trip.
- **Defer the script, or `async` it.** It would run after the first
  paint, and the shift ADR 2026-10-30-style-before-first-paint fixed
  would come back.
- **Preload both files.** They are already the first things the page
  asks for, so a preload can't start them sooner.
- **Inline only the CSS the first screen needs, load the rest later.**
  [web.dev](https://web.dev/articles/extract-critical-css) describes it,
  but applying the rest needs JavaScript or an `onload` attribute, which
  the CSP refuses.
  It would also split one readable stylesheet in two.
- **Serve the static files from the offline worker first, to get round
  the 10-minute lifetime.** It helps only readers on their second page.
  Lighthouse can't see it.
  A stylesheet kept from before a deploy could also mis-draw a newer
  page, which is why ADR 2026-10-29-offline-reading asks the network
  first.
- **A CDN or another host in front of Pages, to set longer lifetimes.**
  Already turned down in ADR 2026-10-28-csp-meta-tag:
  a third party between readers and the site, for a small gain.
