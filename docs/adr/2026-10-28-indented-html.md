# 2026-10-28 — Indented HTML

- Date: 2026-10-28
- Status: Accepted
- Decided by: JC, asking whether view-source of the home page should be
  formatted to be more human readable, "thinking like indentation".

## Context

ADR 2026-10-14-mobile-first-readable-output-lighthouse keeps the output
unminified so readers can audit and remix the site from "view source". The
templates already wrote one element per line, but every line sat flush
left, and template conditionals left stray blank lines. Nesting (header,
nav, the style picker's option groups, list items, the note body) was hard
to follow without the browser's developer tools.

## Decision

- Every HTML page is indented as the last step of rendering
  (`build/indent.rs`): two spaces per open element, blank lines dropped.
  `<html>` adds no level, so `<head>` and `<body>` stay flush left; a line
  that starts with an end tag lines up with its start tag.
- Only whitespace right after a newline changes. Outside `<pre>`,
  `<textarea>`, `<script>` and `<style>` the browser already collapses that
  whitespace into the newline's single space, so pages render exactly as
  before. The contents of those four elements ship byte for byte: code
  blocks stay flush left, because their whitespace is the code.
- Lines are never wrapped or joined. A long paragraph stays on one line;
  breaking it would add whitespace between words and inline elements where
  the author put none.
- Feeds (RSS, JSON Feed) are unchanged; their HTML is escaped text.

## Consequences

- About 650 bytes more per page before compression, about 60 after gzip
  (about 2%). GitHub Pages compresses everything, so this is noise next to
  the fonts.
- The harness checks the shape (`html_is_indented_by_nesting`: head and
  body flush left, `<main>` and `</main>` one level in on every page, which
  also proves every note's HTML closes what it opens) and that every fenced
  code block in the vault ships exactly as written
  (`code_blocks_keep_their_whitespace`). Before merging, screenshots of 7
  pages in 4 styles at phone and desktop width were pixel identical before
  and after.
- A future CSS rule with `white-space: pre` or `pre-wrap` on an element
  other than `<pre>`/`<textarea>` would show the added spaces. Use `<pre>`
  for preformatted text instead.

## Alternatives considered

- **Leave it flush left.** Readable line by line, but the nesting is
  invisible.
- **Indent in the templates.** The note body comes from the Markdown
  renderer, which knows nothing of the page around it, so its lines would
  still need a pass; one pass over the finished page covers both.
- **A full pretty-printer that rewraps text.** Changes rendered spacing
  between inline elements and needs a real HTML parser, against the
  zero-dependency build (ADR 2026-10-06-rust-build-zero-dependencies).
