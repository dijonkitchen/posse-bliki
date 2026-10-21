# 2026-10-21 — Doctor Who replaces Under the Sea

- Date: 2026-10-21
- Status: Accepted
- Decided by: JC, asking that if any of the styles overlap, one of them
  become Doctor Who.

## Context

ADR 2026-10-20-zen-garden-styles brought the picker to sixteen styles.
Laid side by side, three of them are the same idea: deep navy page, near
white text, pale gold links.

| Style | Page | Link |
|---|---|---|
| Blueprint | `#123a66` | `#ffe08a` |
| Under the Sea | `#06304a` | `#ffe27a` |
| Garments | `#1d2f4a` | `#ffcf6e` |

Blueprint and Under the Sea are the closest: their link golds are two
steps apart and both pages are mid navy with a soft texture. Garments
at least has its stitching and sewn-on labels. The only other near pair,
Jekyll and Facebook (both white with dark blue links), differ in layout:
Facebook's full-width blue bar and boxed headings.

## Decision

- **Drop Under the Sea**, the Zen Garden tribute nearest to another style,
  and keep Blueprint (one of the Originals) and Garments.
- **Add Doctor Who** in its place, after the TARDIS: police-box blue run
  across the top of the window, the site title on the black sign above the
  doors with the roof lamp lit (it pulses, unless the reader asks for less
  motion), and the text on the cream roundel wall of the classic console
  room, with the roundels only in the margins. It is light, so it adds a
  look the menu lacked instead of a fourth navy one. Written from scratch:
  no logo, no images, no fonts; Gill Sans and its fallbacks are fonts the
  reader may have.
- **It's seasonal.** It sits in the Seasonal group, and style-picker.js
  picks it for readers who never picked a style on 23 November, the day
  the show first aired in 1963, the same way Diablo and Holiday turn on.
- Everything else from ADR 2026-10-20-zen-garden-styles holds: the sheet
  loads only when picked, it is linked in `<noscript>` so the picker works
  without JavaScript, and its text and link tokens meet AAA 7:1 (the
  harness checks).

## Consequences

- Still sixteen styles, so pages are unchanged in weight: one stylesheet
  and the picker script, and nothing extra unless Doctor Who is picked.
- A reader who had saved Under the Sea finds the picker back on Auto: the
  script ignores a saved value that's no longer in the menu.
- On 23 November a first-time visitor's page loads one extra stylesheet,
  as on the other seasonal days.
- The Zen Garden group is down to five tributes.

## Alternatives considered

- **Replace Garments or Blueprint instead.** Garments is the most textured
  of the three and Blueprint is an Original JC already kept, so either
  would lose more than Under the Sea.
- **Replace Facebook or Jekyll.** They share a palette but not a layout,
  and Jekyll is this site's own history.
- **A dark, time-vortex Doctor Who.** Truer to the modern title sequence,
  but it would be one more dark blue style, the overlap this change
  removes.
