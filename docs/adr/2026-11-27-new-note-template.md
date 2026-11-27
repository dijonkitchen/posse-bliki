# 2026-11-27 — A new note starts from a template, unpublished

- Date: 2026-11-27
- Status: Accepted; amends [2026-05-10-obsidian-as-editor-no-required-plugins](2026-05-10-obsidian-as-editor-no-required-plugins.md) (which plugins the vault carries)
- Decided by: JC, who said "Yes" to a template
  so that a new note starts with `published: false`
  and its checkbox shows up right away on the phone,
  and asked "Does it also work with published at date time".
  Claude chose Templater to do it, for the reasons below.

## Context

A file in `content/` is on the site only while it says `published: true`
(ADR 2026-11-14-published-checkbox).
A new note starts empty,
so to publish one on the phone,
JC first adds the `published` and `title` properties by hand.
The point of a template is that the box is already there, unticked.

Three plugins could put text in a new note:

- **The core Templates plugin** inserts a template
  when you run "Templates: Insert template" from the command palette
  ([Obsidian Help: Templates](https://obsidian.md/help/plugins/templates)).
  Nothing runs it on its own, so it's a step for every note.
- **Linter's "Insert YAML attributes" rule**, in the plugin the vault already carries,
  adds missing keys when Linter lints a note.
  But while you type, Linter runs only its YAML Timestamp rule
  ("When the currently active note is modified, `YAML timestamp` is run on the note",
  in the vault's copy of Linter 1.35.0, `.obsidian/plugins/obsidian-linter/main.js`).
  Its other rules run when a note is saved with Ctrl+S
  (the vault's "Lint on save"; "Lint on focused file change" is off),
  or from its "Lint the current file" command,
  so on the phone, with no Ctrl+S, the box wouldn't show unless you ran that command.
- **Templater's folder templates** fill an empty new file in a folder
  as soon as it is made:
  "You can specify a template that will automatically be used for empty new files created in a selected folder and its children"
  ([Templater: Settings](https://github.com/SilentVoid13/Templater/blob/2.27.0/docs/src/settings.md)).
  It works on the phone (its manifest says `"isDesktopOnly": false`).

## Decision

- **The vault carries Templater 2.27.0**, released 2026-11-08,
  in `.obsidian/plugins/templater-obsidian/`,
  with the code and settings in git like the other plugins
  (ADR 2026-11-14-vault-at-repo-root).
  Its `main.js`, `manifest.json` and `styles.css` are the release's own files
  from [SilentVoid13/Templater](https://github.com/SilentVoid13/Templater/releases/tag/2.27.0),
  the plugin's repo in Obsidian's
  [community plugin list](https://github.com/obsidianmd/obsidian-releases/blob/master/community-plugins.json);
  `main.js` has SHA-256 `0b4a22e7a3871f929e18656d9f3c7defd8588c9dc5845620b9a16e0adb7c9a2f`.
- **Every new, empty note anywhere in `content/` starts from `templates/new-note.md`:**

  ```yaml
  ---
  title:
  published: false
  ---
  ```

  `published: false` shows as an unticked checkbox,
  since "Checkbox properties are either `true` or `false`"
  ([Obsidian Help: Properties](https://obsidian.md/help/properties)),
  and `title` is the one other field a published note must have
  (`spec/content-schema.json`).
  The template has no dates:
  Linter stamps `created_at` and `updated_at` a few seconds after the note changes,
  as it does now (ADR 2026-11-13-created-at),
  and Linter fills a `created_at:` left blank with the file's creation time,
  just as it does a missing one,
  so a blank one in the template would add nothing.
- **Templater touches nothing else.**
  "Only trigger for files matching a template rule" is on,
  so "New files that don't match a rule are left untouched,
  and any Templater commands in them are not run"
  ([Templater: Settings](https://github.com/SilentVoid13/Templater/blob/2.27.0/docs/src/settings.md)),
  and "Templates are applied only to empty files".
  A note that arrives by a pull, with text in it, is left as it is.
- **`templates/` sits outside `content/`,**
  so the build never reads it and Linter never stamps it,
  and `.obsidian/app.json` hides it from search, the graph and link suggestions,
  like the code folders.
- **Each device turns Templater's trigger on once.**
  Templater keeps "Trigger Templater on new file creation"
  in the device's own storage, not in its `data.json`
  (`src/settings/LocalSettings.ts` at 2.27.0),
  so a settings file that came through git can't make it run code.
  On each device: *Settings → Templater → Trigger Templater on new file creation*, then confirm.

### A time to publish

`created_at` is already the time a post goes live,
and no other field is needed.
A published note whose `created_at` is in the future is held back,
and goes out with the first build after that moment:
a push, or the daily rebuild at 06:17 UTC
(ADRs 2026-10-20-scheduled-posts and 2026-11-13-created-at).
So to schedule a post, set `created_at` to a future UTC time and tick the box.

The time has to be typed in UTC, ending in `Z`, such as `2026-12-01T17:00:00Z`.
Obsidian's own Date & time property is stored without a zone,
as `2020-08-21T10:30:00`
([Obsidian Help: Properties](https://obsidian.md/help/properties)).
The build refuses a time without a zone (`spec/content-schema.json`),
and Linter, set to convert to UTC,
reads one as UTC when it next stamps the note
(`parseValueToCurrentFormatIfPossible` in the vault's copy of Linter).
So if Obsidian offers a date picker for `created_at`,
9 AM picked in Pacific time would go out at 9 AM UTC, seven or eight hours early.

## Consequences

- A new note made in Obsidian, on the phone or the laptop,
  opens with an empty `title` and an unticked `published` box,
  and Linter adds its dates a few seconds after the first edit.
  Publishing is filling in the title and ticking the box.
- The build is unchanged, and there is no new build code.
  The cost is about 455 KB of someone else's JavaScript in `.obsidian/`,
  a settings file, a four-line template,
  and a harness test (`tests/harness/obsidian.rs`)
  that a note made from the template is skipped while unticked
  and builds once it has a title and is ticked.
- When a device updates the plugin,
  Obsidian Git commits the new code with a backup,
  as it did for Excalidraw on 2026-11-23.
- Until the trigger is on, a device makes notes without the template,
  which are unpublished, as before.
  So is a note made outside Obsidian.
- Templater skips a new file whose path contains the name of its template folder,
  so a note at a path such as `content/notes/templates.md` starts empty.
- A post is still dated when its file was made:
  a draft started on the 1st and ticked on the 20th is dated the 1st,
  unless `created_at` is changed when the box is ticked.
- Restart Obsidian on each device after pulling this,
  so it loads the new plugin and settings
  (ADR 2026-11-14-vault-at-repo-root).

## Alternatives considered

- **The core Templates plugin.**
  No new code, but every new note needs the "Insert template" command,
  through the command palette or the phone's toolbar.
- **Linter's "Insert YAML attributes" rule.**
  No new plugin, but on the phone the box shows up only
  after running Linter's command by hand.
- **A blank publish time in the template, or a separate `published_at`.**
  `created_at` already holds a post back until its time,
  and JC chose it over `published_at` (ADR 2026-11-13-created-at).
  Two fields would be two dates to keep in step.
- **A "Publish now" command that dates a post when it's ticked.**
  A Templater command, one tap from the phone's toolbar,
  could tick `published` and set `created_at` to the current UTC time,
  so a post is dated when it goes public, in the feeds too, which sort by `created_at`.
  Obsidian has no hook for a checkbox being ticked, so it would be a second way to publish.
  `updated_at` can't stand in: Linter re-stamps it on the tick, but again on every later edit.
  JC chose to keep setting `created_at` by hand when that matters (2026-11-27).
- **Keep adding the properties by hand.**
  Nothing new, but it's the step on the phone this is meant to remove.
