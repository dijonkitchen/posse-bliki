# 2026-10-22 — Share a style with a ?style= link

- Date: 2026-10-22
- Status: Accepted
- Decided by: JC, asking whether a style should be deep-linkable with a
  query parameter for sharing, whether it should stick as people move to
  new pages, and choosing the simpler way: "seems easy enough for someone
  to change their setting back if needed".

## Context

A reader's style lives in their own browser (ADR 2026-10-18-style-picker),
so there was no way to send someone the site in Diablo or Star Trek. A
query parameter is the plain way to put a choice in a link. The open
question was whether a linked style lasts only for the linked page, only
for the visit, or until the reader changes it.

## Decision

- `style-picker.js` reads `?style=<style>`. If the name is one of the
  menu's values, the page opens in that style and it's **saved as the
  reader's pick**, exactly as if they had chosen it from the menu, so it
  stays on across pages and visits until they pick something else.
- A name that isn't in the menu is ignored, and the reader's saved pick
  or the season applies as usual. Only menu values ever reach the
  stylesheet loader, as before.
- The parameter isn't stripped from the address bar or added to the
  site's own links; the saved pick is what carries it from page to page.
- The style garden page says how, with `?style=diablo` as an example in
  code rather than a link: the build checks Markdown links against notes
  and has no notion of a query string, and teaching it one isn't worth it
  for one example.

## Consequences

- One small branch in the existing script; no build change, no new
  request, no new storage.
- A link replaces the reader's own saved style. Picking from the menu
  puts it back, which JC judged easy enough.
- Without JavaScript the parameter does nothing: the picker's no-JS mode
  works through `:has()` on a form control, which a URL can't set.
- Search engines may see `?style=` URLs as the same page; the canonical
  link already points them at the plain address.

## Alternatives considered

- **Only for the linked page.** Nothing saved, but the style drops off on
  the first click, which surprises whoever followed the link.
- **For the visit only** (session storage, or adding the parameter to
  every internal link). Closer to "borrowed", but more code and more
  rules, for a choice the reader can undo in one click.
- **A path or fragment** (`/diablo/...`, `#style=diablo`). Paths would
  duplicate every page; a fragment would clash with heading anchors.
