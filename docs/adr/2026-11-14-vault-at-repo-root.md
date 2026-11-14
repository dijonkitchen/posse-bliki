# 2026-11-14 — The Obsidian vault is the repo root; new notes start in content/notes/

- Date: 2026-11-14
- Status: Accepted; amends [2026-05-10-obsidian-as-editor-no-required-plugins](2026-05-10-obsidian-as-editor-no-required-plugins.md) (where the vault is, and which settings are committed)
- Decided by: JC, who set up the vault at the repo root on phone and laptop
  ("I updated vault to be on phone and laptop and to sync to github regularly", 2026-11-13),
  then asked whether it should open at the root or not,
  and said yes to keeping it there with the settings below (2026-11-14).
  Where new notes go follows from ADR 2026-11-14-published-checkbox:
  JC asked "Should drafts be in same directory",
  then "How about A published Boolean instead", and said "Yes" to it.

## Context

The README said to open `content/` as the vault,
and `content/.obsidian/app.json` held the link settings
([ADR 2026-10-16-relative-markdown-links](2026-10-16-relative-markdown-links.md)).
JC now writes on a phone as well as a laptop,
and syncs both through the Obsidian Git plugin,
which commits and pushes every change straight to `main`.
Every push to `main` deploys.

On a phone the plugin can't reach a repo above the vault.
There it uses its built-in git (isomorphic-git),
whose working directory is the vault root or a "custom base path",
and that setting only accepts "an existing folder in this vault"
(the plugin's own code, `.obsidian/plugins/obsidian-git/main.js`, version 2.43.0;
upstream at [Vinzent03/obsidian-git](https://github.com/Vinzent03/obsidian-git)).
On a laptop the plugin runs the `git` command, which would find the repo from `content/`,
but one layout for both devices is simpler.

So JC opened the repo root as the vault.
That had three side effects:

- Obsidian reads `.obsidian/` at the root, not `content/.obsidian/`,
  so the relative-link setting no longer applied.
  A link written in `content/now.md` to a note could come out as `note.md`,
  which the build rejects as a broken link.
- A new note landed at the repo root, which is never built.
  One created in `content/notes/` would break the deploy on the next backup,
  since every note needed a `title` (tested: "'title' is a required property"),
  until ADR 2026-11-14-published-checkbox let the build skip unpublished notes.
- Search, the graph and link suggestions included the code, specs and ADRs.

## Decision

- The vault is the repo root.
  Its settings live in `.obsidian/`, committed with the plugin code,
  so the phone gets them through git.
  `content/.obsidian/app.json` goes, since nothing reads it.
- `.obsidian/app.json` sets:
  - `newLinkFormat: relative` and `useMarkdownLinks: true`, as before;
  - `alwaysUpdateLinks: true`, so moving a note rewrites its links;
  - `newFileLocation: folder` with `newFileFolderPath: content/notes`,
    so new notes start where they will be published;
  - `userIgnoreFilters` for `build/`, `docs/`, `scripts/`, `spec/`, `target/` and `tests/`,
    so Obsidian's search, graph and link suggestions show only notes.
- A new note stays off the site until its `published` box is ticked,
  and the build skips it until then (ADR 2026-11-14-published-checkbox).
  A note is published by giving it the front matter a published note needs
  (a `title` and its dates, per `spec/content-schema.md`)
  and ticking `published`. The file never moves.
- Linter stamps only `content/`: its "Files to ignore" pattern becomes `^(?!content/)`,
  since there is no `drafts/` folder to stamp
  (ADR 2026-11-12-obsidian-stamps-updated-at).

## Consequences

- An unfinished note can't break the deploy, however often the vault syncs.
- Unpublished notes are still public on GitHub, like the rest of the repo.
- A published note can't link to an unpublished one: the build fails on that link.
- Restart Obsidian on each device after pulling a change to `.obsidian/`,
  so it reads the new settings.
  Until then, changing a setting on that device may write the old ones back.
- The excluded folders still show in the file list,
  and their files open as usual.
- Pasted images still land at the vault root.
  They belong in `content/images/`, linked as `/images/<file>`, by hand for now.

## Alternatives considered

- **Open `content/` as the vault.**
  Obsidian sees only notes, and the old settings keep working.
  Rejected: the phone can't sync it.
- **Clone the repo into a folder inside the vault, with the Git plugin's base path.**
  Also works on a phone, but the vault's `.obsidian/` then sits outside the repo,
  so its settings and plugins don't reach the other device through git.
- **New notes in a `drafts/` folder outside `content/`.**
  The build would never read them, with no field needed.
  Rejected for the `published` checkbox: publishing would mean moving the file,
  and every relative link to or from it with it.
- **New notes in `content/notes/` with `draft: true`.**
  Rejected: every new note needs the front matter typed before the first backup,
  or the deploy breaks; a phone makes that easy to miss.
