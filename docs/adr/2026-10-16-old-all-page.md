# 2026-10-16 — Redirect the old /all page to the notes index

- Date: 2026-10-16
- Status: Accepted
- Decided by: JC, after the site consistency check found `/all` returning
  404.

## Context

The Jekyll site that used to live at www.dijonkitchen.org had an `/all`
page (`all.html`) listing every post title A to Z. After the cutover it
returns 404, so old bookmarks and links to it break. Individual Jekyll
posts already redirect through their aliases. This was the one list page
left without a redirect.

## Decision

`/all` and `/all.html` get the same kind of redirect page as an alias,
pointing at `/notes/`. That's the closest page on the new site, which
lists every note, though newest first rather than A to Z.

`/humans.txt` and Jekyll's `/pageN` pagination are not carried over. The
colophon and the home page's h-card cover what `humans.txt` said, and the
pagination pages were the old home page's own "older posts" links.

## Consequences

- Old links to the post list land on the notes index.
- Like aliases, these redirects are never removed.

## Alternatives considered

- **Leave it 404.** Breaks old links for no saving.
- **Rebuild an A-to-Z page at `/all/`.** A second index that does almost
  the same job as `/notes/`; worth revisiting only if the order matters.
