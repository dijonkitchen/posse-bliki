# 0005 — Obsidian as editor, no required plugins

- Date: 2026-05-10
- Status: Accepted

## Context

The vault must be editable by a tool that supports wikilinks, backlinks,
and search at the editor level — Obsidian is the obvious fit. But
Obsidian has a deep plugin ecosystem; depending on plugins for content
correctness would mean the vault only renders correctly inside Obsidian.

## Decision

The vault uses **only standard markdown plus Obsidian's native wikilink
and tag syntax**. No required community plugins. The build does not
read any Obsidian-specific files (`.obsidian/`, canvases, etc.) — they
are ignored.

Personal Obsidian setup (themes, hotkeys, plugins) is the user's choice
and lives in `.obsidian/` (mostly gitignored).

## Consequences

- Notes open identically in any markdown editor: VS Code, Logseq, Helix,
  vim, GitHub's web view.
- The vault survives Obsidian itself disappearing.
- Build is simpler: it only knows about `.md` files.
- Cost: features that some Obsidian users rely on (Dataview queries,
  templater, etc.) are inert in the published site. By design.

## Alternatives considered

- **Allow Dataview.** Tempting — it makes index pages dynamic. Rejected:
  it ties content correctness to a plugin. If we want auto-generated
  index pages, do it in `build/` from the same data Dataview would use.
