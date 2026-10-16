# 2026-10-16 — A style garden, one page per style

- Date: 2026-10-16
- Status: Proposed
- Decided by: JC, asking whether a CSS theme selector like the old CSS Zen
  Garden would be worth a little fun.

## Context

The [CSS Zen Garden](https://csszengarden.com) showed one fixed HTML page in
many designs, each a single stylesheet. That fits this site's view-source
ethos (ADR 2026-10-14-mobile-first-readable-output-lighthouse): stylesheets
ship unminified so readers can read and remix them.

The site has no JavaScript (spec/output-contract.md), no `<style>` blocks
and no inline styles (spec/design.md), and GitHub Pages can't set cookies.
So nothing can remember a reader's choice from one page to the next. The
ways to switch styles without JavaScript were:

- **`rel="alternate stylesheet"`.** Only Firefox offers a menu for it
  (View > Page Style), the choice resets on every page, and other browsers
  still download the alternates.
- **A `:has()` radio picker.** Radio buttons in every page's header, with
  rules like `:root:has(#terminal:checked)`. Works in current browsers, but
  the choice resets on every link a reader follows, which feels broken
  site-wide, and every theme's CSS lands in `style.css` for every reader.
- **One page per style.** The garden page is built once per style, each copy
  linking `/style.css` plus one file from `/themes/`. Plain links switch
  between them. It works in every browser, and each style stays its own file.

## Decision

- Add a style garden at `/garden/` (from `content/garden.md`). The build also
  writes `/garden/<style>/` for each style in `build/static/themes/`, linking
  `/style.css` and then `/themes/<style>.css`. A "Styles" list on each copy
  links to all of them and marks the current one with `aria-current`.
- Styles apply only to the garden. The rest of the site keeps the house
  style and follows `prefers-color-scheme`.
- Each style is a short, commented, unminified file that loads after the
  house stylesheet and changes tokens, fonts and a few flourishes. It loads
  nothing (no `@import`, no `url()`), and its text pairs meet WCAG AAA 7:1,
  like the house style. Decorative generated text is hidden from screen
  readers with the `content: "…" / ""` alt syntax.
- Three styles to start: Terminal, Broadsheet and Blueprint.

## Consequences

- Adding a style is a new CSS file plus one line in `THEMES` in
  `build/site.rs`. The harness checks each file in `build/static/themes/` has
  its garden page and is linked from every other one.
- Readers who like a style can't keep it while reading notes. That is the
  cost of having no JavaScript and no cookies, and the garden page says so.
- This doesn't settle the light and dark toggle question. A site-wide toggle
  still needs JavaScript to remember the choice.

## Alternatives

- `rel="alternate stylesheet"` and a site-wide `:has()` picker, for the
  reasons above.
- Building the whole site once per style (`/terminal/notes/…`), so a choice
  carries through links. It multiplies the output and the sitemap's
  duplicate pages for a bit of fun.
