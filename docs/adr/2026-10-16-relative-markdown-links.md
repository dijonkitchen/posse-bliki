# 2026-10-16 — Relative Markdown links between notes

- Date: 2026-10-16
- Status: Accepted
- Decided by: JC, who asked for links that read well on GitHub and in
  Obsidian and work on the site, and chose "Markdown links" over keeping
  wikilinks.
- Amends: [2026-05-10-obsidian-as-editor-no-required-plugins](2026-05-10-obsidian-as-editor-no-required-plugins.md),
  which named Obsidian's wikilink syntax as the way notes link.

## Context

`content/` is read in three places: Obsidian (where it's written), GitHub
(where it's stored and reviewed) and the built site. GitHub doesn't render
`[[wikilinks]]` in repository files; they show as raw brackets. Links
written as site URLs (`/freedom`) work on the site only, through alias
redirect pages, and are broken on GitHub and in Obsidian. The content had
both, plus one relative link (`./process` in `notes/vision.md`) that was
broken on the site and that the link check didn't catch, because it only
looked at root-absolute hrefs.

## Decision

- Notes link to each other with standard Markdown links to the file,
  relative to the linking note: `[text](other-note.md)`,
  `[text](../about.md#heading)`. That path works on GitHub and in Obsidian
  as written.
- The build rewrites each relative `.md` href to the linked note's URL
  (`/notes/other-note/`) after rendering, slugs the `#heading` the same
  way wikilinks do, and fails on a link to a file that isn't a published
  note (missing, a draft, or outside `content/`).
- The committed Obsidian settings (`content/.obsidian/app.json`) turn on
  Markdown links with the relative path format, so typing `[[` in
  Obsidian still autocompletes but writes this form.
- The build keeps resolving `[[wikilinks]]`, so a note promoted from the
  private Modin vault (which uses wikilinks) still builds. The harness
  fails on wikilinks or site-URL links in `content/`, so they get converted
  when a note is promoted. Modin itself is unchanged (JC: "Don't modify
  Modin for now").
- The link check now resolves relative hrefs too, not only `/…` ones.

## Consequences

- Every note link is clickable on GitHub, in Obsidian and on the site.
- `.md` links feed backlinks like wikilinks do, since both end up as note
  URLs before backlinks are collected.
- Links carry paths, so moving a note changes the links to it. Obsidian
  rewrites them on rename (`alwaysUpdateLinks`); outside Obsidian the build
  fails on any link left behind.
- Images still use `/images/…`, which works on the site but not on GitHub.
  That's a separate change if wanted.

## Alternatives considered

- **Keep wikilinks.** Best typing experience in Obsidian and no paths to
  maintain, but GitHub shows raw brackets. Rejected because GitHub
  readability was part of the ask.
- **Convert Modin too.** One script turns its wikilinks into Markdown
  links. Deferred at JC's request; promotion converts notes one at a time.
- **Extension-less links (`[x](other-note)`).** Obsidian resolves them, but
  GitHub doesn't. Rejected.
