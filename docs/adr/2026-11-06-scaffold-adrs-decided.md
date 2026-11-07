# 2026-11-06 — Record who decided the scaffold's ADRs

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, answering "Update" when told that the May ADRs name no
  decider and that the Obsidian one still says to use wikilinks.

## Context

The five ADRs dated 2026-05-10 came with the first scaffold
([PR #1](https://github.com/dijonkitchen/posse-bliki/pull/1)). Claude wrote
them along with the code, and JC merged the PR 20 seconds after it opened.
None of them says who decided. Every ADR since then has a "Decided by"
line, so these five looked like JC's decisions when they were really
proposals that JC accepted by merging.

JC has since made later decisions that settle each of them, but nothing
pointed from the old ADRs to those. ADRs are append-only, and the status
line is the only part of an accepted ADR that may change
(docs/adr/README.md). So this ADR records the record, and each old ADR's
status line links here.

## Decision

The scaffold ADRs stand as follows:

| ADR | Proposed by | Where it stands, and the decisions by JC that settle it |
|---|---|---|
| 2026-05-10-spec-and-harness-over-implementation | Claude, PR #1 | Accepted. JC kept the spec and harness as the lasting part when choosing the zero-dependency Rust build (2026-10-06-rust-build-zero-dependencies) and the Rust-only toolchain (2026-10-11-rust-only-toolchain). |
| 2026-05-10-no-quartz-no-vendored-ssg | Claude, PR #1 | Accepted. JC chose a custom build over any SSG in 2026-10-06-rust-build-zero-dependencies. Its "working wikilinks" need was replaced by relative Markdown links (2026-10-16-relative-markdown-links). |
| 2026-05-10-uv-for-python-toolchain | Claude, PR #1 | Superseded by JC's 2026-10-11-rust-only-toolchain. Unchanged. |
| 2026-05-10-indieweb-standards-supported | Claude, PR #1 | Accepted, as amended by JC: webmentions sent and fetched at deploy time (2026-10-11-webmentions-at-deploy-time), the IndieWeb Webring (2026-11-06-indieweb-webring) and no Pingback (2026-11-06-drop-pingback). |
| 2026-05-10-obsidian-as-editor-no-required-plugins | Claude, PR #1 | Accepted for "Obsidian, no required plugins". Its "native wikilink and tag syntax" is out: notes link with relative Markdown paths (2026-10-16-relative-markdown-links) and have no tags (2026-11-06-drop-tags). |

Each of these ADRs' status lines now links here. Their bodies are left as
they were.

From now on, every ADR's header includes "Decided by", which names who
made the call and how. docs/adr/README.md says so, so a proposal nobody
chose can't pass as a decision again.

## Consequences

- Reading any of the five shows where it stands, without needing to know
  the later history.
- The old bodies still describe the May plan (Python, wikilinks, tags).
  The status line and this table say what changed.

## Alternatives considered

- **Add a "Decided by" line to each old ADR.** It's more direct, but it
  edits accepted ADRs beyond their status lines, which the ADR rules
  forbid.
- **Supersede each one with a new ADR.** That's five new ADRs that would
  restate decisions already recorded elsewhere.
