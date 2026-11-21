# 2026-11-21 — Search page that searches in the browser

- Date: 2026-11-21
- Status: Accepted
- Decided by: JC, asking "Is there a simple search that can be made",
  then choosing this page (recommended)
  over a no-JS hand-off to DuckDuckGo or no search for now.

## Context

The site has 67 notes and a few pages, listed on `/notes/` by title,
with no way to find a note by what it says.
A static site on GitHub Pages has no server to run a query,
so a search either runs in the reader's browser
or is handed to a web search engine.

Constraints it has to live within:

- The site works with JS off,
  and its one script so far is the style picker
  (ADR 2026-10-18-style-picker).
- The CSP allows only the site's own files,
  with `form-action 'none'` and the two inline blocks by hash
  (ADR 2026-10-28-csp-meta-tag, ADR 2026-11-11-inline-house-style-and-script).
- No dependencies: the build, tests and tools are Rust's standard library
  (ADR 2026-10-06-rust-build-zero-dependencies).
- Output is readable, never minified (ADR 2026-10-14-mobile-first-readable-output-lighthouse),
  and extras cost a page nothing until used (ADR 2026-10-20-zen-garden-styles).
- Pages are kept for reading offline (ADR 2026-10-29-offline-reading).
- JC maintains the site alone and wants the code simple.

## Decision

- **`/search/`**, a page with a search form, linked last in the header's nav.
  Its box and button are `disabled` in the HTML,
  so a reader with JS off never gets a box that does nothing;
  under it a `<noscript>` note says search needs JavaScript
  and points to Notes.
- **`/search.json`**, written by the build:
  the URL, title and plain text of every note, in the order of `/notes/`,
  then the other pages.
  About 120 KB, 45 KB gzipped today.
  It's its own file rather than `/feed.json`,
  which leaves out evergreen notes and pages and carries HTML
  (spec/feeds.md); writing it is about 20 lines of build code.
- **`/search.js`**, about 120 readable lines (a fifth of them comments),
  loaded `defer` by that page only.
  It enables the form, fetches `search.json` beside itself
  (via [`document.currentScript`](https://html.spec.whatwg.org/multipage/dom.html#dom-document-currentscript),
  so it works under a sub-path)
  the first time the reader searches,
  and filters it as they type:
  every word must start a word of the title or text,
  ignoring case and accents.
  Title matches come first, then the order of Notes.
  Each result shows the text around the first match, marked.
  Results are built from text nodes, never parsed as HTML.
  The words go in the address as `?q=` with `history.replaceState`,
  so a search can be shared or reloaded.
- The script is a file of the site's own,
  which `script-src 'self'` already allows,
  and it never submits the form,
  so the CSP is unchanged,
  [`form-action 'none'`](https://www.w3.org/TR/CSP3/#directive-form-action) included.
- The service worker keeps `/search/` and `/search.js` on install,
  about 10 KB, so the page opens offline,
  and keeps `search.json` once read, so search works offline after one search.
- `/search/` joins Lighthouse's fixed pages,
  as the one page with a form and its own script.

## Consequences

- Search is instant, always matches what's published,
  and sends what the reader types nowhere.
- The site has a second script, on one page.
  Every other page is unchanged but for the nav link
  and a few lines of house style (shared with the select and list rules).
- With JS off there is no search;
  Notes and the browser's Find stay the way to look.
- `search.json` grows with the notes, about 0.7 KB gzipped per note,
  and loads only when someone searches.
  A few hundred notes still fit in one fetch;
  far past that, an index split by word would be worth it.
- Matching is plain word starts, with no stemming or typo tolerance:
  "write" finds "writing" but not "wrote".
- `tests/harness/search.rs` checks the form, the index
  (every built note and page, in order, as plain text, nothing unpublished),
  that the script writes no HTML, fetches only its index and never submits,
  and its syntax; `base_path.rs` checks it under a sub-path.

## Alternatives considered

- **Hand off to DuckDuckGo.**
  A form sending `site:www.dijonkitchen.org` and the words to DuckDuckGo
  ([search syntax](https://duckduckgo.com/duckduckgo-help-pages/results/syntax)).
  About 15 lines and no JS,
  but `form-action` would have to allow DuckDuckGo,
  readers leave the site, their query goes to a third party,
  it doesn't work offline,
  and a new note shows up only once it's crawled.
- **Search `/feed.json`.** No build code,
  but the feed leaves out evergreen notes and pages,
  and its HTML would need parsing in the browser.
- **Pagefind, Lunr or another prebuilt search.**
  Ranked, fuzzy search, but a dependency to install or vendor
  and much more JS.
  Pagefind searches with WebAssembly,
  which needs `'wasm-unsafe-eval'` in `script-src`
  ([Pagefind: hosting](https://pagefind.app/docs/hosting)).
  More than 67 notes need.
- **Include every note's text in the search page,**
  so the browser's Find searches it with JS off.
  No script, but a page of hundreds of kilobytes.
