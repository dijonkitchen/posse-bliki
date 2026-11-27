//! The vault's committed Obsidian settings stamp `created_at` and
//! `updated_at` in the form the build reads, and only in notes
//! (spec/content-schema.md, ADRs 2026-11-12-obsidian-stamps-updated-at and
//! 2026-11-13-created-at), and start a new note from a template the build
//! accepts (ADR 2026-11-27-new-note-template). The vault is the repo root,
//! so its settings are the root `.obsidian/`.

use std::fs;

use posse_bliki_build::date::Stamp;
use posse_bliki_build::json::{self, Json};

use crate::support::{build, read_as_built, repo, temp_dir, vault_copy};

fn settings(file: &str) -> Json {
    let path = repo().join(".obsidian").join(file);
    json::parse(&read_as_built(&path)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

fn linter() -> Json {
    settings("plugins/obsidian-linter/data.json")
}

fn timestamp_rule(data: &Json) -> &Json {
    data.get("ruleConfigs")
        .and_then(|r| r.get("yaml-timestamp"))
        .expect("no yaml-timestamp rule")
}

fn is(rule: &Json, key: &str, want: &str) -> bool {
    rule.get(key) == Some(&Json::Lit(want.to_string()))
}

#[test]
fn linter_is_turned_on_in_the_vault() {
    let on = settings("community-plugins.json");
    assert!(
        on.as_arr().iter().any(|p| p.as_str() == Some("obsidian-linter")),
        "community-plugins.json doesn't list obsidian-linter"
    );
}

#[test]
fn linter_stamps_both_dates_in_utc() {
    let data = linter();
    let rule = timestamp_rule(&data);
    assert!(is(rule, "enabled", "true"), "the YAML Timestamp rule is off");
    assert!(is(rule, "date-modified", "true"), "the modified date is off");
    assert_eq!(rule.str_at("date-modified-key"), Some("updated_at"));
    assert!(is(rule, "convert-to-utc", "true"), "stamps would be in local time");
    // Every note gets a created_at once, from its file's creation time,
    // and the value in the note stands after that (ADR 2026-11-13-created-at).
    assert!(is(rule, "date-created", "true"), "the created date is off");
    assert_eq!(rule.str_at("date-created-key"), Some("created_at"));
    assert_eq!(
        rule.str_at("date-created-source-of-truth"),
        Some("frontmatter"),
        "Linter would overwrite created_at with the file's time on each device"
    );
    // What the format writes for one moment is something the build reads.
    let format = rule.str_at("format").expect("no format");
    let written = [
        ("YYYY", "2026"),
        ("MM", "11"),
        ("DD", "12"),
        ("HH", "04"),
        ("mm", "27"),
        ("ss", "28"),
        ("[Z]", "Z"),
    ]
    .iter()
    .fold(format.to_string(), |s, (token, value)| s.replace(token, value));
    assert_eq!(
        Stamp::parse(&written).map(|s| s.iso()).as_deref(),
        Some("2026-11-12T04:27:28Z"),
        "format {format:?} writes {written:?}"
    );
}

#[test]
fn linter_stamps_only_notes() {
    // Linter tests each pattern as a JavaScript RegExp against the file's
    // path from the vault root, and skips the files that match.
    let data = linter();
    let ignored: Vec<&str> = data
        .get("filesToIgnore")
        .map(|f| f.as_arr().iter().filter_map(|p| p.str_at("match")).collect())
        .unwrap_or_default();
    assert!(
        ignored.contains(&"^(?!content/)"),
        "Linter would stamp README.md, the specs and the ADRs, or skip notes: filesToIgnore is {ignored:?}"
    );
}

/// The template Templater writes into a new, empty note in `content/`.
fn new_note_template() -> String {
    let data = settings("plugins/templater-obsidian/data.json");
    assert_eq!(
        data.str_at("trigger_on_file_creation_mode"),
        Some("folder"),
        "new notes wouldn't get the template"
    );
    // Templater then runs nothing in any other new file, such as a note
    // pulled from git that happens to contain `<% ... %>`.
    assert_eq!(
        data.get("only_trigger_for_matching_files"),
        Some(&Json::Lit("true".to_string())),
        "Templater would run commands in every new file"
    );
    let rules = data.get("folder_templates").map(Json::as_arr).unwrap_or_default();
    let template = rules
        .iter()
        .find(|r| r.str_at("folder") == Some("content"))
        .and_then(|r| r.str_at("template"))
        .expect("no folder template for content/");
    read_as_built(&repo().join(template))
}

#[test]
fn a_new_note_starts_unticked_and_builds_once_ticked() {
    let on = settings("community-plugins.json");
    assert!(
        on.as_arr().iter().any(|p| p.as_str() == Some("templater-obsidian")),
        "community-plugins.json doesn't list templater-obsidian"
    );
    let template = new_note_template();
    assert!(
        template.contains("\npublished: false\n"),
        "the template doesn't start unpublished:\n{template}"
    );
    // A new note as Linter leaves it a few seconds later, with its stamps
    // at the end of the front matter (the first "\n---\n" closes it).
    let stamped = template.replacen(
        "\n---\n",
        "\ncreated_at: 2026-11-27T17:40:00Z\nupdated_at: 2026-11-27T17:40:05Z\n---\n",
        1,
    ) + "\nhalf a thought\n";
    let ticked = stamped
        .replace("title:\n", "title: From the template\n")
        .replace("published: false", "published: true");
    let vault = vault_copy("new-note-template");
    fs::write(vault.join("notes/unticked.md"), &stamped).unwrap();
    fs::write(vault.join("notes/ticked.md"), &ticked).unwrap();
    let out = temp_dir("new-note-template-out");
    build(&vault, &out, &["--today", "2026-11-28"]).expect("a note from the template builds");
    assert!(
        !out.join("notes/unticked/index.html").exists(),
        "an unticked note was built"
    );
    assert!(
        out.join("notes/ticked/index.html").exists(),
        "a ticked note wasn't built"
    );
}
