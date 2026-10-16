# Design

How pages look. The visual language is JC's **a11y Design System** (a
Design System artifact on claude.ai): maximally readable for people of all
ages, including people with low vision, colour-vision deficiency, dyslexia
or tremor. Values in `build/static/style.css` are copied from the system's
`tokens/*.css`; when the system changes, copy the new values across.
See [ADR 2026-10-11-a11y-design-system-self-hosted-fonts](../docs/adr/2026-10-11-a11y-design-system-self-hosted-fonts.md).

## Contract

Each item is enforced by `tests/harness/design.rs`.

- **One stylesheet.** Every page links `/style.css`. No inline `style`
  attributes, no `<style>` blocks.
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
  measure 66ch. Sizes are in `rem` so the reader's own text-size setting
  scales the page.
- **Header.** The site title and tagline sit in the header outside `<nav>`;
  the nav holds only links. On the home page the title is the `<h1>` at h1
  size; elsewhere it is a smaller link and the page has its own `<h1>`
  (ADR 2026-10-16-site-title-heads-home).
- **Targets.** Navigation links and tags are at least 44px tall.
- **Mobile first.** The phone layout is the default; see
  [performance.md](performance.md).

## Not yet

The system's reader settings (text size, spacing, weight, line length,
theme toggle) need JavaScript to remember a choice. The site is JS-free
(see [output-contract](output-contract.md)), so pages follow the operating
system's colour scheme and the browser's text size instead.
