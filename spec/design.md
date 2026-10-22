# Design

How pages look. The visual language is JC's **a11y Design System** (a
Design System artifact on claude.ai): maximally readable for people of all
ages, including people with low vision, colour-vision deficiency, dyslexia
or tremor. Values in `build/static/style.css` are copied from the system's
`tokens/*.css`; when the system changes, copy the new values across.
See [ADR 2026-10-11-a11y-design-system-self-hosted-fonts](../docs/adr/2026-10-11-a11y-design-system-self-hosted-fonts.md).

## Contract

Each item is enforced by `tests/harness/design.rs`.

- **Stylesheets.** Every page's `<head>` links `/style.css`, then, inside
  `<noscript>`, each style in `build/static/themes/` as
  `/themes/<style>.css`. With JavaScript on, the page loads `/style.css`
  alone and `/style-picker.js` adds only the picked style's sheet, so a
  style costs nothing until someone picks it
  (ADR 2026-10-20-zen-garden-styles). No inline `style` attributes, no
  `<style>` blocks.
- **Style picker.** Every page's header has a Style menu (`<select
  id="style">`): Auto, Light, Dark, then each theme in an `<optgroup>`
  (Originals, Editors, CSS Zen Garden, Seasonal, Throwbacks). Picking one
  restyles the page through CSS `:has()`, with no JavaScript;
  `/style-picker.js` loads the picked style's sheet and remembers the pick
  across pages. Auto follows `prefers-color-scheme`. A theme's rules all
  sit inside `:root:has(#style [value="<style>"]:checked)`, it loads
  nothing itself, ships as written, and its text and link tokens meet AAA
  7:1 on every surface (ADR 2026-10-18-style-picker).
- **Share links.** A link ending `?style=<style>` opens the page in that
  style and saves it as the reader's pick, as if chosen from the menu, so
  it stays on across pages until they pick another. A name not in the menu
  is ignored. It needs the script; without it the page shows the reader's
  usual style (ADR 2026-10-22-style-share-links).
- **Seasonal styles.** A reader who has never picked a style gets Star
  Trek on 8 September, Diablo from 24 to 31 October, Doctor Who on 23
  November, Holiday from 1 to 26 December, and on 1 April any style in the
  menu at random, drawn again on each page, by their own clock. It isn't
  saved, and picking anything, Auto included, overrides it (ADR
  2026-10-20-zen-garden-styles, ADR 2026-10-21-doctor-who-style, ADR
  2026-10-22-star-trek-style, ADR
  2026-10-22-ft-solarized-and-editor-styles).
- **Tributes, not copies.** Styles after other people's designs (the CSS
  Zen Garden ones, Diablo, Doctor Who, Star Trek, Broadsheet's FT, the
  editor themes) are written from scratch: no stylesheets, images or fonts
  from the originals, and each file credits the design it's after.
- **Round shapes stay round.** A circle drawn with `radial-gradient` gives
  its stops in lengths or is sized `closest-side`, so its tile never cuts
  it square.
- **No third-party requests.** Pages and the stylesheet load nothing from
  another origin: no web-font CDNs, no `@import`. Fonts are self-hosted
  under `/fonts/`, with their SIL OFL licences beside them.
- **Works under a sub-path.** Every `url()` in the stylesheet is relative to
  the stylesheet and resolves to an emitted file, so the site renders the
  same at a domain root and at a GitHub Pages project URL.
- **Design tokens.** The stylesheet defines the system's semantic colour
  tokens (`--text-1`, `--text-2`, `--text-link`, `--surface-page`,
  `--surface-sunken`, `--border-strong`, `--border-subtle`, `--focus-halo`,
  `--focus-ring`) for light, and redefines them under
  `@media (prefers-color-scheme: dark)`. Rules use semantic tokens, not
  primitives.
- **User preferences honoured.** The stylesheet responds to
  `prefers-color-scheme`, `prefers-reduced-motion`, `prefers-contrast` and
  `forced-colors`.
- **Always-visible focus.** `:focus-visible` draws the system's yellow halo
  plus ink ring (`--focus-outline`), with a real outline in forced colours.
- **Header.** Every page's header shows the site title, linking home, with
  the tagline under it (ADR 2026-10-14-site-and-author-naming).
- **Skip link.** The first element in `<body>` is a link to `#main`
  ("Skip to content"), and `<main id="main">` exists on every page.
- **Readable type.** Atkinson Hyperlegible Next for text, JetBrains Mono for
  code, 18px (`1.125rem`) body, nothing below 16px, line-height 1.6, text
  measure 66 characters of Atkinson (`42.77em`). Sizes are in `rem` so the
  reader's own text-size setting scales the page. Before the web font
  arrives, text renders in Verdana or Arial resized to Atkinson's metrics,
  so nothing jumps when it swaps in
  (ADR 2026-10-20-metric-matched-fallback-fonts).
- **Header.** The site title and tagline sit in the header outside `<nav>`;
  the nav holds only links. On the home page the title is the `<h1>` at h1
  size; elsewhere it is a smaller link and the page has its own `<h1>`
  (ADR 2026-10-16-site-title-heads-home).
- **Browser bars match the page.** Every page declares `theme-color` for
  light and dark, set to `--surface-page`, so a phone's browser bars blend
  into the page and it reads full bleed. `<html>` has the page background
  too, so scrolling past the top or bottom shows the page colour
  (ADR 2026-10-16-browser-bars-match-page).
- **Targets.** Navigation links and tags are at least 44px tall.
- **Mobile first.** The phone layout is the default; see
  [performance.md](performance.md).

## Not yet

The system's other reader settings (text size, spacing, weight, line
length) aren't offered. The browser's own text size still scales the page.
