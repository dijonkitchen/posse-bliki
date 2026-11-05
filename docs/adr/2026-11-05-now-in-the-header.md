# 2026-11-05 — Now in the header, Colophon in the footer

- Date: 2026-11-05
- Status: Accepted
- Decided by: JC, asking "should now page be on top bar" once /now went
  live; Claude recommended swapping it in for Colophon.

## Context

[ADR 2026-11-01-now-page](2026-11-01-now-page.md) linked the now page
from the footer, to leave the header alone. Most now pages are linked
from the main menu, because "what is this person up to" is one of the
first things a visitor looks for, next to who they are.

The header nav was Notes, About, Colophon, and every footer already
links the colophon. A fourth header link, checked at phone widths in
all 19 styles plus Light and Dark:

- at 390px (most phones) it fits on one line everywhere but Star Trek,
  where Colophon wraps onto its own row;
- at 320px (small phones) Broadsheet, Tranquille, Steel, Jekyll and
  Facebook wrap Colophon onto its own row, and Star Trek wraps to three.

With three links nothing wraps at 390px, and only Star Trek does at
320px, as it already did.

## Decision

- The header nav is **Notes, Now, About**. Now appears only when the now
  page is published; without it the nav is Notes, About.
- Colophon leaves the header and stays in every footer with the feeds
  and the blogroll. Now leaves the footer, since it's in the header.
- The offline worker's install shell keeps the header's pages, so it
  keeps `now/` instead of `colophon/`. A harness test checks every
  header link is in the shell, since a missing file fails the whole
  install.

## Consequences

- Readers find /now/ from every page in one tap, in every style.
- The colophon is one scroll further away; readers who want "how is
  this built" tend to look in the footer anyway.
- Deleting `content/now.md` drops the header link, but `sw.js` still
  lists `now/`; the harness's shell test fails until it's removed there
  too.
- Star Trek's LCARS pills colour links by position (`nth-child`), so the
  three links keep its lavender/orange pattern.

## Alternatives considered

- **Add Now as a fourth link.** Keeps Colophon a tap away, at the cost
  of a lone wrapped link on small phones in six styles and on most
  phones in Star Trek; fixing that means tighter nav spacing in each of
  those styles.
- **Keep Now in the footer only.** No layout change, but the page is
  easy to miss, and it isn't where readers expect it.
