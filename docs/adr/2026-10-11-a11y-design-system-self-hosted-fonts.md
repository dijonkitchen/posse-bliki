# 2026-10-11 — a11y design system, self-hosted fonts

- Date: 2026-10-11
- Status: Accepted
- Decided by: JC, asking for the site to match their design system.

## Context

The first stylesheet was 40 lines of placeholder styles: serif body, sans
headings, four colours. JC has a design system, **a11y**, built for maximum
readability: Atkinson Hyperlegible Next, an off-white page with near-black
ink, AAA (7:1) text contrast, 18px body, 44px+ targets, a yellow-and-ink
focus ring, and automatic dark mode. The system loads its fonts from Google
Fonts and says to self-host them in production.

## Decision

- Restyle the site with the system's tokens and element defaults in the one
  stylesheet, `build/static/style.css`. [spec/design.md](../../spec/design.md)
  states what the harness checks.
- Self-host the fonts: latin-subset variable WOFF2 files of Atkinson
  Hyperlegible Next (roman and italic) and JetBrains Mono, with their SIL OFL
  1.1 licences, in `build/static/fonts/`. The binary embeds them with
  `include_bytes!` and writes them to `fonts/`, the same way it embeds
  `style.css`. About 100 KiB in all.
- Stay JS-free. The system's reader-settings panel is left out because it
  needs a script to persist choices.

## Consequences

- Pages make no third-party requests, so readers aren't tracked by a font
  CDN and the site works offline and where Google Fonts is blocked.
- The binary grows by the font bytes. The build stays std-only (ADR 2026-10-06-rust-build-zero-dependencies).
- Characters outside the latin subset fall back to the system font stack.
- When the design system changes, its values are copied into `style.css` by
  hand; there is no sync.

## Alternatives considered

- **Load fonts from Google Fonts.** Simplest, but a third-party request on
  every page and a privacy cost for readers.
- **System fonts only.** No bytes to ship, but loses the hyperlegible
  typeface that is the core of the system.
- **Ship the reader-settings script.** Useful, but it breaks the JS-free
  output contract; it would need its own ADR.
