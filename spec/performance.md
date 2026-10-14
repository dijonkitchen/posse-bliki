# Performance

How pages load. The site should be quick on a phone on a slow connection,
score at the top of Chrome Lighthouse, and still be pleasant to read with
"view source". See [ADR 0014](../docs/adr/0014-mobile-first-readable-output-lighthouse.md).

## Contract

Each item is enforced by `tests/harness/performance.rs`, except the Lighthouse
budget, which `scripts/lighthouse.sh` checks in CI.

- **Mobile first.** Rules outside any width query are the phone layout.
  Wider screens add to it with `@media (min-width: …)`; the stylesheet has no
  `max-width` query. On phones the heading scale is one step smaller and the
  page gutter is 16px; from `40rem` up they are the design system's full
  scale and 24px.
- **Readable output, never minified.** `style.css` ships byte for byte as
  written in `build/static/`, comments included, with no line over 200
  characters. HTML keeps one `<head>` element per line, as the templates
  write it. GitHub Pages compresses everything over the wire, so a minifier
  would save almost nothing.
- **Every indexable page has a description.** `<meta name="description">`
  comes from the note's `summary`, else the first 160 characters of its
  text, cut at a word; the home page uses its `summary` or the site tagline;
  list and tag pages say what they list. Alias redirects carry `noindex`
  and need none.
- **An icon.** Every page links `/favicon.svg`, so browsers don't request
  `/favicon.ico` and log a 404.
- **Images are self-hosted and sized.** Images live in `content/images/`,
  are referenced as `/images/<file>` and are copied to the output as is.
  The build reads each one's pixel size from its header (PNG, GIF, JPEG,
  WebP) and writes `width` and `height`, so nothing shifts as it loads. A
  missing image is a build error. Keep photos at most about 800px wide and
  well compressed before adding them.
- **Only what's needed loads first.** The first image on a page may be the
  largest thing on screen, so it loads eagerly; later images load with
  `loading="lazy"`; all decode with `decoding="async"`. Embeds (`<iframe>`)
  always load lazily and scale to the screen at 16:9.

## Lighthouse budget

`make lighthouse` (CI-only, like `make validate`) builds and serves the site
and runs Lighthouse's default mobile audit on the home page, a post with
code, a post with a large photo, a post with a video embed, `/notes/` and
`/about/`. It fails if any page scores below:

| Category | Minimum |
|---|---|
| Performance | 90 |
| Accessibility | 100 |
| Best practices | 100 |
| SEO | 100 |

Performance has headroom because shared CI runners are noisy; the others
are pass/fail checks. Locally every page scores 100 in all four.

Lighthouse is locked, with every transitive package, in
`scripts/lighthouse/package-lock.json` and installed with `npm ci`, so CI
runs exactly the reviewed versions.

## PRPL

How the [PRPL pattern](https://web.dev/articles/apply-instant-loading-with-prpl)
maps onto a static, script-free site:

- **Push / preload** the critical resources: the body font is preloaded,
  the one stylesheet is same-origin and small, and there are no
  third-party requests to set up.
- **Render** the initial route as soon as possible: every page is complete
  HTML from the server, with no client rendering.
- **Pre-cache** remaining routes: not done. It needs a service worker,
  which is JavaScript, and the site has none
  ([output-contract](output-contract.md)). The browser's HTTP cache keeps
  the stylesheet and fonts between pages.
- **Lazy-load** everything else: images after the first, and embeds.

## Outside the build

These count in Lighthouse on the live site but are set by the host, not by
the build:

- GitHub Pages serves everything with `Cache-Control: max-age=600`, which
  Lighthouse flags as a short cache lifetime. Pages has no setting for it.
- The address in `base_url` should be the one Pages serves without a
  redirect; a redirect from the bare domain to `www` (or back) costs every
  first visit a round trip.
