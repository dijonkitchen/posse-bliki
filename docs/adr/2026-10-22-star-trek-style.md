# 2026-10-22 — Fewer blues, and a Star Trek style

- Date: 2026-10-22
- Status: Accepted
- Decided by: JC, on seeing ADR 2026-10-21-doctor-who-style: "you don't
  need the other blue ones then", and "also add a star trek one for
  space".

## Context

ADR 2026-10-21-doctor-who-style replaced Under the Sea, one of three deep
navy styles with pale gold links, and kept the other two, Blueprint and
Garments. JC would rather not keep them either. Space was Angelfire's
backdrop alone, and Angelfire is a 1990s home page, not a ship.

## Decision

- **Remove Blueprint and Garments.** The Originals are now Terminal and
  Broadsheet; the CSS Zen Garden group keeps Tranquille, Mid Century,
  Steel and Apothecary. A reader who had saved either finds the picker
  back on Auto, as with Under the Sea.
- **Add Star Trek**, after the ship's computer screens of The Next
  Generation: black glass; rounded orange and lavender bars bent into
  elbows round the header and footer; header links as pill buttons in
  black on orange, lavender and periwinkle; tall narrow capitals for
  headings from fonts the reader may have (Helvetica Neue Condensed, Arial
  Narrow and so on), with body text left readable; and stars outside the
  text column, which stays solid black so text over them keeps AAA.
  Written from scratch: no insignia, logos or fonts.
- **It's seasonal**, like Doctor Who: style-picker.js picks it for readers
  who never picked a style on 8 September, Star Trek Day, the day the
  show first aired in 1966.
- The rules of ADR 2026-10-20-zen-garden-styles still hold: it loads only
  when picked, is linked in `<noscript>` for readers without JavaScript,
  and its text and link tokens meet AAA 7:1 (the harness checks).

## Consequences

- Fifteen styles instead of sixteen. Pages are no heavier for readers
  with JavaScript (still one stylesheet and the script), and one
  stylesheet lighter without it.
- There is no navy style left. Readers who liked Blueprint's graph paper
  or Garments' denim lose them; both are in git history to restore.
- On 8 September a first-time visitor's page loads one extra stylesheet,
  as on the other seasonal days.

## Alternatives considered

- **Keep one of the two blues.** Blueprint was the last navy and an
  Original, but JC said neither is needed.
- **A starship exterior or a starfield style.** More "space", but a
  starfield alone is close to Angelfire, and the computer screens are
  what reads as Star Trek at a glance.
