# 2026-10-20 — Zen Garden, seasonal and throwback styles, loaded only when picked

- Date: 2026-10-20
- Status: Accepted; Under the Sea replaced by Doctor Who in
  [ADR 2026-10-21-doctor-who-style](2026-10-21-doctor-who-style.md), and
  Garments removed by [ADR 2026-10-22-star-trek-style](2026-10-22-star-trek-style.md);
  April Fools' Day draws from every style since
  [ADR 2026-10-22-ft-solarized-and-editor-styles](2026-10-22-ft-solarized-and-editor-styles.md)
- Decided by: JC, asking to "have some fun and duplicate the same styles as
  CSS Zen Garden" without affecting page performance, then for a Diablo I
  style that turns on for Halloween, a holiday style for Christmas time,
  and Xanga, GeoCities, Angelfire and Jekyll throwbacks; then a classic
  Facebook throwback, with the throwbacks coming up on April Fools' Day.

## Context

ADR 2026-10-18-style-picker linked every style's stylesheet from every
page, so each one was free to apply but not free to load: with three
styles a page made four render-blocking stylesheet requests. Adding thirteen
more that way would have added thirteen more requests to every page for
every reader, almost all of whom never open the menu.

Measured with `scripts/lighthouse.sh` (Lighthouse mobile, median of three,
same six pages as CI) before this change: performance 99 to 100, four
stylesheets and five render-blocking requests per page.

The CSS Zen Garden designs belong to their designers, and their art is
images and commercial fonts. Copying the stylesheets or assets would break
both their licences and this site's rules (no `url()`, no font downloads,
AAA text).

## Decision

- **Load only the picked style.** Pages link `/style.css` alone. The theme
  stylesheets are linked inside a `<noscript>` in `<head>`, so with
  JavaScript off the picker still works through `:has()` exactly as before.
  With JavaScript on, `/style-picker.js` adds the picked style's sheet,
  marked `blocking="render"` so a remembered style shows without a flash,
  and adds a sheet the first time a reader picks it from the menu. Adding a
  style now costs nothing to a page that doesn't use it.
- **Thirteen new styles**, grouped in the menu with `<optgroup>`:
  - *CSS Zen Garden*: Tranquille (Dave Shea), Mid Century (Andrew Lohman),
    Steel (Steffen Knoeller), Apothecary (Trent Walton), Under the Sea
    (Eric Stoltz) and Garments (Dan Mall). Each is written from scratch in
    the spirit of the original: no images, no fonts, shapes drawn with CSS
    gradients, and the header comment credits the design it's after.
  - *Seasonal*: Diablo, after the first Diablo (1996), and Holiday.
  - *Throwbacks*: GeoCities, Angelfire, Xanga, Jekyll and Facebook.
    Jekyll is after this site's own first home, the Jekyll default theme
    of dijonkitchen.github.io; Facebook is the 2007 profile page, with no
    logo. Both darken their original blue a little to reach AAA.
  - The existing Terminal, Broadsheet and Blueprint become *Originals*.
- **Seasons.** A reader who has never picked a style gets Diablo from 24 to
  31 October, Holiday from 1 to 26 December, and on 1 April a random
  throwback, drawn again on every page, by their own clock. It isn't
  saved, so the site goes back to Auto after the season, and picking
  anything, Auto included, overrides it. The dates live in
  `style-picker.js`; there is no build-time switch, because the site only
  rebuilds on a push.
- **Contrast is tested.** A new harness test checks every style's text and
  link tokens against each surface at WCAG AAA 7:1. Decorations that would
  sit behind text (Diablo's life and mana orbs, Mid Century's circles,
  Under the Sea's bubbles) only show on screens wide enough to keep them in
  the margins; snow and stars are kept off the text column.

## Consequences

- Measured after, same method: performance 99 to 100 on every page, one
  stylesheet and two render-blocking requests per page (the stylesheet
  and the picker script), and about 4 KB less transferred per page, with
  sixteen styles instead of three.
- A reader with a picked style makes one more request for it, cached
  between pages. With JavaScript off, a page loads every style, seventeen
  stylesheets, as the old design did with four; that's the price of the
  picker working without a script.
- In browsers without `blocking="render"` (Firefox and Safari as of
  writing) a remembered style may draw a frame of the house style first.
- During the seasons, a first-time visitor's page loads one extra
  stylesheet.
- Fonts named by the styles (Futura, Baskerville, Comic Sans and so on) are
  ones the reader may have; each has a fallback, and nothing is downloaded.

## Alternatives considered

- **Keep linking every style.** Simplest, but seventeen render-blocking
  requests on every page, which JC ruled out.
- **One `/themes.css` with every style.** One request instead of sixteen,
  but every reader downloads every style, and a style stops being one file
  to read and remix.
- **Seasonal styles switched at build time.** No script, but the site only
  rebuilds on a push, so it would need a scheduled deploy, and it would
  ignore the reader's own time zone.
- **Copying the Zen Garden stylesheets and images.** Faithful, but not
  ours to copy, and it would bring in images and fonts.
