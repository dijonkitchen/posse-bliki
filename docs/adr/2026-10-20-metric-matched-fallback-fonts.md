# 2026-10-20 — Metric-matched fallback fonts

- Date: 2026-10-20
- Status: Accepted
- Decided by: JC, asking whether Medium's 2015 advice to use system fonts
  still holds, then choosing to keep Atkinson Hyperlegible Next and only add
  size-matched fallbacks.

## Context

Medium's design team made the case in 2015 for system UI font stacks: no
font bytes, no swap, native look. Today the long stack they published
(`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, …`) is mostly
`system-ui`, and Medium itself now serves web fonts for reading.

This site already weighed system-only fonts and kept Atkinson Hyperlegible
Next, because it is the core of JC's a11y design system
(ADR 2026-10-11-a11y-design-system-self-hosted-fonts). Its cost is small: a
34 KB preloaded roman file, cached between pages. The cost that remains is
the swap. With `font-display: swap`, text first paints in Verdana or Arial,
then reflows when Atkinson arrives. Verdana is about 15% wider, so lines
rewrap and the page jumps (layout shift).

CSS can resize a local font to match a web font: `size-adjust` scales its
glyphs, and `ascent-override`, `descent-override` and `line-gap-override`
set its vertical metrics. A fallback tuned this way takes the same space as
Atkinson, so the swap changes the letterforms but not the layout.

## Decision

- Add `@font-face` rules for two local-only families, `Atkinson Fallback
  Verdana` (`local("Verdana")`) and `Atkinson Fallback Arial`
  (`local("Arial")`, with Liberation Sans and Arimo, its metric twins on
  Linux and ChromeOS), each with a regular (200–600) and a bold (700–800)
  face.
- `size-adjust` is Atkinson's average latin glyph width over the local
  font's, from Capsize's `xWidthAvg` metrics (the same method as Next.js
  and Fontaine). The ascent and descent overrides are Atkinson's (984 and
  316 units per 1000) divided by that, and the line gap is 0, as in
  Atkinson.
- They sit straight after Atkinson in `--font-sans`, Verdana first to keep
  the stack's order. The plain system fonts stay behind them for systems
  with neither.
- The text measure moves from `66ch` to `42.77em`, the same width with
  Atkinson loaded (its "0" is 0.648em). `ch` is the width of the current
  font's "0", so in `ch` the column narrowed by up to a fifth while a
  fallback showed, which rewrapped every line and undid the fix on wide
  screens. Terminal and Broadsheet keep `66ch`, since their fonts are
  their own.
- `tests/harness/design.rs` checks the order and that each fallback face is
  local-only and sets all four overrides.

## Consequences

- No new bytes or requests: `local()` fonts are already on the reader's
  machine.
- Less layout shift while fonts load, most of all on a first visit, when
  the font isn't cached yet.
- If Atkinson never loads (blocked, offline), readers get Verdana or Arial
  at Atkinson's size rather than at its own. That's intended: the page's
  rhythm stays the same.
- Italics have no fallback face, so the browser slants the roman face
  until the italic file arrives. That's a brief state and not worth four
  more faces.
- The numbers are tied to these font files. If the design system changes
  font, recompute them.

## Alternatives considered

- **System fonts only (Medium's advice).** No bytes or swap, but it drops
  the hyperlegible typeface that is the point of the design system, and the
  type would differ on every OS. Rejected again, as in
  ADR 2026-10-11-a11y-design-system-self-hosted-fonts.
- **`font-display: optional`.** No shift, but a first-time reader on a slow
  connection would get the fallback for the whole visit.
- **Drop JetBrains Mono for the system monospace stack.** It would save
  31 KB on pages with code, but it would change how the Terminal and
  Blueprint styles look. JC chose not to.
