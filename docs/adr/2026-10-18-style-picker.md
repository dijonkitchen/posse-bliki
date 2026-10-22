# 2026-10-18 — A style picker on every page, with one small script

- Date: 2026-10-18
- Status: Accepted; how styles load superseded by [2026-10-20-zen-garden-styles](2026-10-20-zen-garden-styles.md);
  Blueprint removed by [2026-10-22-star-trek-style](2026-10-22-star-trek-style.md)
- Decided by: JC, asking for a CSS Zen Garden style theme selector, then
  for it to be a dropdown like a light and dark mode picker, with minimal
  JavaScript so the choice works across pages, and without needing
  JavaScript to work on one page.

## Context

The [CSS Zen Garden](https://csszengarden.com) showed one fixed HTML page in
many designs, each a single stylesheet. That fits this site's view-source
ethos (ADR 2026-10-14-mobile-first-readable-output-lighthouse): stylesheets
ship unminified so readers can read and remix them.

The site had no JavaScript, and the light and dark question was open: the
site followed `prefers-color-scheme`, and a toggle had been held back because
remembering a choice needs JavaScript. GitHub Pages can't set cookies, so
nothing else can carry a choice from page to page.

A `<select>` can drive CSS on its own: `option:checked` matches the selected
option, and `:root:has(#style [value="dark"]:checked)` matches the page while
Dark is picked. So the picker works on the current page with JavaScript off,
in every browser with `:has()`; browsers without it keep the house style.

## Decision

- Every page's header has a Style menu: Auto, Light, Dark, then Terminal,
  Broadsheet and Blueprint. Auto follows the operating system, as before.
  Light and Dark pin the house palette; Dark repeats the dark tokens (a
  harness test keeps both copies equal).
- Each extra style is a commented, unminified file in `build/static/themes/`,
  served at `/themes/<style>.css` and linked from every page after
  `/style.css`. All its rules sit inside one
  `:root:has(#style [value="<style>"]:checked)` selector using CSS nesting,
  so it costs nothing until picked. It loads nothing itself and keeps text at
  WCAG AAA 7:1, like the house style.
- One script, `/style-picker.js`, about 20 lines and unminified, remembers
  the pick in `localStorage` and restores it on the next page. It is a
  plain `<script src>` right after the picker, not deferred, so the
  remembered style applies before the rest of the page is drawn. It is the
  only script on the site, and the site works fully without it.
- `/garden/` is a page to try the styles on, with a bit of every element a
  note uses.

## Consequences

- The no-`<script>` rule in spec/output-contract.md now allows this one
  script and nothing else; the harness checks that.
- Every page makes three more small stylesheet requests. The Lighthouse
  budget in CI decides whether that is acceptable.
- With JavaScript off, a pick lasts for the current page only.
- Browsers without CSS nesting or `:has()` show the house style and a menu
  that does nothing.
- `prefers-contrast: more` (secondary text at full strength) applies under
  Auto and Light only. Pinned Dark and the extra styles keep their own
  secondary text, which still meets AAA.
- `theme-color` still follows the operating system, so with a pinned style
  the phone's browser bars can differ from the page.
- Adding a style is a new CSS file plus one line in `THEMES` in
  `build/site.rs`.

## Alternatives

- **One page per style** (`/garden/<style>/`), the first draft. No
  JavaScript at all, but a pick couldn't follow the reader, and JC wanted a
  site-wide dropdown.
- **`rel="alternate stylesheet"`.** Only Firefox offers a menu for it, and
  the choice resets on every page.
- **Radio buttons with `:has()`.** Works the same as the menu, but takes far
  more room in the header than a dropdown.
- **Putting every style in `style.css`.** One request instead of four, but
  each style stops being one file to read and remix.
