# 2026-10-16 — Browser bars match the page colour

- Date: 2026-10-16
- Status: Accepted
- Decided by: JC, asking whether the phone view could be full bleed instead
  of showing contrasting bands at the top and bottom.

## Context

The page has no header or footer background: the page colour already runs
edge to edge. The bands JC saw on a phone are the browser's own address bar
and toolbar. Without a `theme-color`, each browser picks its own colour for
them, which can differ from the page colour, especially in dark mode.

## Decision

- Every page's `<head>` declares two `<meta name="theme-color">` tags, one
  for `(prefers-color-scheme: light)` and one for dark, each set to
  `--surface-page` written as hex (`#fbfaf7` and `#15171b`), because
  `theme-color` cannot read CSS variables.
- `<html>` has the page background as well as `<body>`. JC saw a different
  colour in dark mode when scrolling past the top or bottom of a page; some
  phone browsers paint that overscroll area from the root element's own
  background rather than the one `<body>` passes up to it.
- The web app manifest keeps `theme_color` and `background_color` as the light
  page colour. A manifest has no dark variant, and it is only used when the
  site is installed.

## Consequences

- Browsers that honour `theme-color` paint their bars in the page colour, so
  the page reads full bleed. Browsers that ignore it are unchanged.
- If `--surface-page` changes, both hex values in `build/site.rs` change with
  it. A harness test checks that the light value matches the stylesheet.

## Alternatives considered

- **Leave it.** The bars stay whatever colour the browser picks.
- **A coloured header and footer band.** The opposite of what JC asked for.
