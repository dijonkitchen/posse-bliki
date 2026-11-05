# 2026-10-29 — Offline reading with a service worker

- Date: 2026-10-29
- Status: Accepted
- Decided by: JC, asking whether the site should become a PWA for offline
  reading, then choosing "Yes, cache as you read" over precaching the
  whole site, and asking to precache the main pages but not every note.

## Context

The site already ships a web manifest and icons (spec/performance.md), so
the one piece of a PWA it lacked was a service worker, which is what makes
pages readable with no network. The browser's HTTP cache doesn't: GitHub
Pages serves everything with `max-age=600`, and an offline browser won't
reuse a stale copy of a page.

Constraints the worker has to live within:

- JavaScript stays minimal and optional (ADR 2026-10-18-style-picker);
  the site works with JS off.
- Styles and other extras must not cost a page anything up front
  (ADR 2026-10-20-zen-garden-styles), and Lighthouse has a budget
  (ADR 2026-10-14-mobile-first).
- Output is never minified (ADR 2026-10-14-mobile-first).
- The CSP meta tag allows only the site's own scripts
  (ADR 2026-10-28-csp-meta-tag). A worker's script is governed by
  `worker-src`, which falls back to `script-src 'self'`, so `/sw.js` is
  already allowed and the policy is unchanged.
- The site may be served under a sub-path (spec/output-contract.md).

The whole site is about 1 MB, more than half of it images.

## Decision

- **`/sw.js`**, a readable, hand-written service worker of about 70 lines,
  copied to the output unchanged like `/style-picker.js`.
- **Started by `/style-picker.js`**, once the page has finished loading,
  so it never competes with drawing the page. No second `<script>` tag.
  Registration failing (private windows, blocked storage) is ignored.
- **Network first.** Every same-origin GET goes to the network, and a 200
  response is copied into the cache. Only when the network fails does the
  worker answer from the cache. Online readers therefore always get the
  live page, and there is no cache version to bump on deploy. Navigation
  preload starts the page request while the worker boots, so it adds no
  wait online.
- **Cache as you read.** On install the worker keeps only a small shell:
  the pages in the header (home, `/notes/`, `/about/`, `/colophon/`, so
  the notes index is there to pick from), `/offline.html`, `style.css`,
  `style-picker.js`, the body font and the tab icon (about 55 KB over the
  wire, most of it the font). Every other page, image, font and picked
  style is kept the first time it's read.
- **`/offline.html`**, a normal site page saying the page isn't saved yet,
  shown for an unread page while offline. It's `noindex`, has no canonical
  and isn't in the sitemap.
- Offline, a `?style=` link falls back to the kept copy of the same page.
- Other hosts and non-GET requests pass straight through.
- Paths in the worker are relative to it, so it works under a sub-path.

## Consequences

- Pages a reader has opened stay readable on a plane or underground, in
  the style they picked.
- The site now runs two scripts, though still only one `<script>` tag.
  With JS off nothing changes.
- The cache has no size limit or expiry. At about 1 MB for the whole
  site, a reader can't fill it; the browser evicts it under storage
  pressure. If the site grows a lot, a cap may be needed.
- On a very slow connection ("lie-fi") a page waits for the network
  before falling back, the same as without a worker.
- Not installable as an app yet: browsers want the manifest's `display`
  to be `standalone` or `minimal-ui`, and it is `browser`. That's a
  separate choice about how the site looks once installed.
- `tests/harness/offline.rs` checks the shell's files exist (a missing
  one would fail the install silently), that the worker is network first
  and same-origin only, that the picker starts it, and that the offline
  page stays out of search.

## Alternatives considered

- **Precache the whole site.** Every post offline after one visit, but
  about 1 MB on a first visit, against the page-performance rule.
- **Cache first with a versioned cache.** Faster repeat visits, but risks
  showing stale posts and needs the build to stamp a version on each
  deploy.
- **No worker.** The HTTP cache covers quick revisits but not offline.
