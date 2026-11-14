# 2026-11-14 — A `published` checkbox puts a note on the site

- Date: 2026-11-14
- Status: Accepted
- Decided by: JC, who asked "Should drafts be in same directory",
  then "How about A published Boolean instead".
  Claude had recommended a separate `drafts/` folder,
  then recommended the checkbox over it, as below; JC: "Yes".

## Context

JC writes in Obsidian on the phone and the laptop,
and Obsidian Git pushes every save to `main`, which deploys
(ADR 2026-11-14-vault-at-repo-root).
So a note is on its way to the site from its first keystroke.

Until now a note was on the site unless it said `draft: true`,
and the build checked every note's front matter, drafts included.
A new note, made with no front matter or only what Linter stamps,
has no `title`, so it failed the build and stopped the deploy
("'title' is a required property"), even with `draft: true`.

Keeping unfinished notes in a `drafts/` folder outside `content/` would fix that,
but publishing would then mean moving the file,
and every relative link to or from it would have to move with it.

## Decision

- **A file in `content/` is on the site only while its front matter says `published: true`.**
  Obsidian's Checkbox property type is "either `true` or `false`",
  and "displays as a checkbox" in Live Preview
  ([Obsidian Help: Properties](https://obsidian.md/help/properties)),
  so publishing is ticking a box.
  Jekyll, which ran the old site, has the same field the other way round:
  "Set to false if you don't want a specific post to show up when the site is generated"
  ([Jekyll: Front Matter](https://jekyllrb.com/docs/front-matter/)).
  Here the default is off, because a note starts unfinished.
- **The build skips every other file before checking anything else.**
  No page, no listing, no feed item, no sitemap entry, no alias, no webmention,
  and no check of its fields or its links,
  so a half-written note can't break the deploy.
  A link to it from a published note is a broken link, as a link to a draft was.
  Front matter that isn't YAML at all still fails the build,
  since then the build can't tell whether the file is published.
- **`published` must be `true` or `false`.**
  A quoted `"true"` is a string and fails the build,
  so a typo can't quietly hide a note.
- **`draft` goes away.**
  Every file in `content/` gets `published: true` once, in this change,
  so the site is the same before and after.
  A published file that still has `draft` fails the build and says to drop it,
  rather than leaving two fields that say opposite things.
- **New notes go in `content/notes/`,** where they will be published,
  not in a `drafts/` folder (ADR 2026-11-14-vault-at-repo-root).

## Consequences

- Publishing is one tick, on the phone or the laptop,
  and the file never moves, so its links never change.
- A note that should be live but was never ticked is silently left out.
  That's the cost of the default being off;
  the push still shows the note in the repo, and the site doesn't list it.
- The checks a note must pass (title, dates, links) apply at the moment it's ticked,
  so a broken link shows up when publishing, not while writing.
- Every file in `content/` carries one more line of front matter.
- If Obsidian shows `published` as text rather than a checkbox,
  setting the property's type to Checkbox once fixes it,
  since "all properties with that name across your vault will use the same type"
  ([Obsidian Help: Properties](https://obsidian.md/help/properties)).

## Alternatives considered

- **A `drafts/` folder outside `content/`** (Claude's first recommendation).
  It keeps unfinished notes out of the build without a field,
  but publishing means moving the file and fixing its links.
- **Keep `draft: true`, and skip checking drafts.**
  Smaller, but a new note has no front matter,
  so it would be live, and failing, until someone added `draft: true`.
  The safe default needs the field to say yes, not no.
- **Skip invalid notes instead of failing.**
  A published note with a typo would vanish from the site without a word.
