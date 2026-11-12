# 2026-11-12 — Obsidian stamps `updated_at` on save

- Date: 2026-11-12
- Status: Accepted
- Decided by: JC ("be sure to setup those community plugins if needed"),
  after asking "Can't obsidian auto update an updated_at property on save?"
  and Claude recommending Linter over Update time on edit.

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
  has a YAML Timestamp rule that stamps a note when it is linted.
  Linting can be tied to saving with Ctrl+S
  ([Running rules](https://platers.github.io/obsidian-linter/usage/running-rules/)),
  which Obsidian's own autosave doesn't trigger.
  The key and the format are settings,
  and "Convert local time to UTC" writes the stamp in UTC.

ADR 2026-05-10-obsidian-as-editor-no-required-plugins keeps content
correct without any plugin, and leaves the plugins a person uses to them.

## Decision

- **The vault's committed Obsidian settings turn on Linter, with one rule.**
  `content/.obsidian/community-plugins.json` lists it,
  and `content/.obsidian/plugins/obsidian-linter/data.json` turns on YAML Timestamp alone:
  the modified date under the key `updated_at`,
  in the format `YYYY-MM-DDTHH:mm:ss[Z]`, converted to UTC,
  with no created date, linting on save.
- **Only the settings are in the repo.**
  Each device installs the plugin's code from Obsidian's community directory,
  and `.gitignore` keeps that code out.
- **The plugin stays optional.**
  The build reads the field, whoever wrote it.
  An edit made anywhere else, on GitHub's site or by an agent in a PR,
  sets `updated_at` by hand when it reworks a note's text,
  and leaves it alone for a typo, a formatting fix or a change across many notes
  (AGENTS.md).

## Consequences

- JC installs Linter once on each device.
  Obsidian runs in Restricted mode by default,
  so community plugins are turned on in its settings first
  ([Obsidian Help: Plugin security](https://obsidian.md/help/plugin-security)).
- Saving with Ctrl+S says "I tended this".
  An edit that Obsidian autosaves, with no Ctrl+S, leaves the stamp alone,
  so a quick typo fix needn't move a note up.
- To stamp on every edit instead,
  set the rule's "Update YAML timestamp on file contents update".
- Linter's "Lint all files" command would stamp every note at once.
  The README says not to run it.
- Linter fills `data.json` with the defaults of its other rules, all off, the first time it runs.
  That fuller file is committed once.
- The settings were written from Linter 1.35.0's source.
  `tests/harness/obsidian.rs` checks that what they write is what the build reads.

## Alternatives considered

- **Update time on edit.**
  Less to set up, since its key already defaults to `updated`,
  but it stamps on changes from outside Obsidian and adds `created`.
- **Commit the plugin's code too**, so opening the vault is the whole setup.
  It would put a megabyte of third-party JavaScript, bundled past reading,
  into a repo whose output is meant to be read (ADR 2026-10-14-mobile-first-readable-output-lighthouse),
  and have it run on JC's devices from a diff nobody can review.
  Installing from Obsidian's directory leaves that trust decision with JC.
- **Only describe the settings in the README.**
  Six settings to click on every device, free to drift from what the build reads.
- **A bot that writes the field after each push.**
  It would need write access to `main`,
  which the workflows are denied on purpose (ADR 2026-10-14-supply-chain-pinned-actions),
  and it couldn't tell a rework from a typo fix any better than git dates can.
