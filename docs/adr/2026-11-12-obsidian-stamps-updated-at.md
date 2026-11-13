# 2026-11-12 — Obsidian stamps `updated_at` as you edit

- Date: 2026-11-12
- Status: Accepted
- Decided by: JC ("be sure to setup those community plugins if needed"),
  after asking "Can't obsidian auto update an updated_at property on save?"
  and Claude recommending Linter over Update time on edit.
  On 2026-11-13 JC opened the repo root as the vault,
  committed Linter's code with it,
  and set its YAML Timestamp rule to stamp five seconds after an edit.

## Context

`updated_at` says when a note was last reworked
(ADR 2026-11-12-updated-at-utc-datetimes).
JC wanted the editor to write it, not a person.

Obsidian has nothing built in for this:
its [Properties](https://obsidian.md/help/properties) have types, not values that fill themselves in.
Two community plugins do it.

- [Update time on edit](https://github.com/beaussan/update-time-on-edit-obsidian)
  stamps a note on every save.
  Its README says
  "If file change from an external source, the header keys will be updated",
  so a pull that changes a note would stamp it too,
  and changes made across many notes would move them all up.
  It also writes a `created` key, which the content schema has no field for.
- [Linter](https://platers.github.io/obsidian-linter/settings/yaml-rules/#yaml-timestamp)
  has a YAML Timestamp rule that stamps a note when it is linted,
  which can be on save
  ([Running rules](https://platers.github.io/obsidian-linter/usage/running-rules/)),
  and, with "Update YAML timestamp on file contents update",
  a few seconds after an edit in the editor.
  The key and the format are settings,
  and "Convert local time to UTC" writes the stamp in UTC.

ADR 2026-05-10-obsidian-as-editor-no-required-plugins keeps content
correct without any plugin.

A first version of this ADR put Linter's settings in `content/.obsidian/`,
for a vault opened at `content/`,
and left each device to install the plugin's code.
On 2026-11-13 JC opened the repo root as the vault on phone and laptop instead,
and Obsidian Git now commits the whole vault, plugin code included, to `main`.
Obsidian reads only the vault root's `.obsidian/`,
so the settings in `content/.obsidian/` went unread.
JC set the rule up by hand there,
in Linter's default format, `dddd, MMMM Do YYYY, h:mm:ss a`,
which the build can't read,
so the first stamp stopped the deploy (fixed by #105).

## Decision

- **The vault's settings at the repo root turn Linter's YAML Timestamp rule on.**
  `.obsidian/plugins/obsidian-linter/data.json` keeps JC's settings
  and sets the rule to the form the build reads:
  the modified date under the key `updated_at`,
  in the format `YYYY-MM-DDTHH:mm:ss[Z]`, converted to UTC,
  with no created date.
- **It stamps five seconds after an edit, and on save.**
  JC's setting, "after 5 seconds", stamps a note
  once its text has stopped changing in the editor,
  which works on the phone, where there's no Ctrl+S.
  Lint on save stays on.
- **Only files in `content/` are stamped.**
  The vault is now the whole repo,
  and the README, specs and ADRs are Markdown too.
  Linter's "Files to ignore" holds one pattern, `^(?!content/)`,
  which matches every path outside `content/`,
  so no rule touches those files.
- **The plugin's code is committed with the vault.**
  JC's vault carries it, as Obsidian Git commits it,
  so the phone and the laptop run the same version with no install step.
- **The plugin stays optional for the build.**
  The build reads the field, whoever wrote it.
  An edit made anywhere else, on GitHub's site or by an agent in a PR,
  sets `updated_at` by hand when it reworks a note's text,
  and leaves it alone for a typo, a formatting fix or a change across many notes
  (AGENTS.md).

## Consequences

- Any edit made in Obsidian stamps the note, a typo fix included,
  and moves it up Home and Notes.
  Edits in the 24 hours after a post is dated aren't counted as updates
  (ADR 2026-11-12-updated-at-utc-datetimes),
  so writing and polishing a new post doesn't.
- The rule listens for edits in the editor and for saves,
  so a pull doesn't stamp a note that isn't open.
  Whether Obsidian reports a pull into an open note as an edit hasn't been checked.
- Linter's "Lint all files" command would stamp every note in `content/` at once.
  The README says not to run it.
- Linter reads its settings when Obsidian starts,
  so a change to them takes effect once Obsidian is reopened on each device.
- About a megabyte of bundled third-party JavaScript is in the repo,
  updated whenever Linter updates on one of JC's devices.
  The build never loads it.
- The rule's behaviour was read from Linter 1.35.0's source.
  `tests/harness/obsidian.rs` checks that what the format writes is what the build reads,
  and that files outside `content/` are ignored.

## Alternatives considered

- **Update time on edit.**
  Less to set up, since its key already defaults to `updated`,
  but it stamps on changes from outside Obsidian and adds `created`.
- **Stamp on save only** ("Update YAML timestamp on file contents update" off),
  as the first version of this ADR had it,
  so a quick typo fix needn't move a note up.
  The phone has no Ctrl+S, so notes edited there would never be stamped.
- **Install the plugin's code on each device**, as the first version had it.
  Keeps third-party code out of the repo,
  but JC's vault commits it, and the phone needs it there to sync.
- **A bot that writes the field after each push.**
  It would need write access to `main`,
  which the workflows are denied on purpose (ADR 2026-10-14-supply-chain-pinned-actions),
  and it couldn't tell a rework from a typo fix any better than git dates can.
