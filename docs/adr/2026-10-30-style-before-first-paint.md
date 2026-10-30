# 2026-10-30 — The picked style is in place before first paint

- Date: 2026-10-30
- Status: Accepted
- Decided by: JC, choosing to fix the Halloween style shifting pages as
  they load, in its own PR.

## Context

From 24 to 31 October, readers who never picked a style get Diablo
(ADR 2026-10-20-zen-garden-styles). Lighthouse kept failing `/about/` on
PR #63 with a layout shift of 0.351 in some runs. The report showed
`themes/diablo.css` arriving after the first paint. The page drew in the
house style, then Diablo changed the header's height and `<main>` moved
down. With Playwright, delaying `diablo.css` by 400 ms gives the same
0.351 shift every time. Real readers see it too, on any page, whenever the
stylesheet is slower than the first paint. That covers every seasonal
style and anyone with a saved style on a cold cache.

`style-picker.js` already added the sheet with `blocking="render"`, but
the script ran inside `<header>`, right after the picker. A browser only
honours `blocking="render"` for elements added while it is still parsing
`<head>`. Once `<body>` has started, the sheet loads like any other, and
the page may paint without it.

Each theme's rules matched only the picker,
`:root:has(#style [value="<style>"]:checked)`. So the script couldn't run
any earlier, because the picker didn't exist yet.

## Decision

- `<script src="/style-picker.js" data-styles="…">` moves to the end of
  `<head>`. `data-styles` lists the menu's styles, built from the same
  list as the menu, since the picker isn't parsed yet.
- The script chooses the style as before: a `?style=` link, then the
  saved pick, then the season. It sets `<html data-style="<style>">` and
  adds the sheet. From `<head>`, `blocking="render"` holds the first paint
  until the sheet arrives.
- Every theme's scope, and Light and Dark in `style.css`, become
  `:root:is([data-style="<style>"], :has(#style [value="<style>"]:checked))`.
  The `:is()` takes the specificity of its strongest part, so the cascade
  is unchanged.
- On `DOMContentLoaded` the script sets the picker to the style in use and
  listens for changes. A change updates `data-style` as well as saving.
- `tests/harness/design.rs` checks that the script is in `<head>` with the
  menu's styles, and that every theme's scope has both parts.

## Consequences

- No layout shift when a style arrives. With `diablo.css` delayed 400 ms,
  `/about/` and `/` shift 0, down from 0.351. Locally, Lighthouse scores
  100 in every category on `/`, `/about/` and `/notes/`, with Diablo on.
- Saved Light and Dark are also in place at first paint, not only after
  the header is parsed.
- With JavaScript off, nothing changes: no `data-style`, and `:has()`
  follows the picker as before.
- The script now holds up the first paint while it loads. It is about
  4 KB unminified, served from the same host and cached between pages. A
  theme that turns on also holds up the first paint until its sheet
  arrives. That's the trade the old `blocking="render"` already intended.
- The menu's styles appear twice in each page, in the menu and in
  `data-styles`. Both come from one list in the build, and a test keeps
  them equal.

## Alternatives considered

- **Hide the page until the sheet loads.** Keeps the script in the
  header, but the header may already have painted, so it would flash,
  and it needs a timeout of its own.
- **Inline the seasonal styles in `style.css`.** No late sheet, but every
  reader would pay for styles they rarely see. That breaks
  ADR 2026-10-20-zen-garden-styles.
- **A second, tiny script in `<head>`.** Same effect, but two scripts and
  two requests instead of one.
