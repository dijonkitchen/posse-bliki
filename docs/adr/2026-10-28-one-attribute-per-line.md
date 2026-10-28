# 2026-10-28 — One attribute per line

- Date: 2026-10-28
- Status: Accepted
- Decided by: JC, after indented HTML shipped (ADR 2026-10-28-indented-html),
  asking that "view source" be "more readable with indentation on tags and
  their properties", for lines like
  `<a class="u-url p-name" href="/notes/wonder/">wonder</a>`.

## Context

Indenting by nesting made the page structure visible, but a tag with
several attributes still packed them onto one line: the theme-color and
CSP `<meta>` tags in `<head>`, the microformat links on every list and note,
images with their sizes. Attributes are where most of the information is,
and the longest lines on a page were these tags.

## Decision

- A tag with two or more attributes puts its name on the line where it
  starts and each attribute on its own line below, one level (two spaces)
  deeper than the line the tag starts on. The `>` closes the last attribute
  line, so the element's content follows right after it, as before:

  ```html
  <a
    class="u-url p-name"
    href="/notes/wonder/">wonder</a>
  ```

- A tag with one attribute, or none, stays on one line (`<a href="/">`):
  splitting it adds a line without making anything easier to find.
- A tag that starts partway through an attribute line (a `<span>` inside a
  split `<a>`) indents one level deeper than that line, so each tag's
  attributes sit under it.
- It is part of the indenting pass (`build/indent.rs`), which already reads
  each tag whole, and it never touches the contents of `<pre>`,
  `<textarea>`, `<script>` or `<style>`.
- Whitespace between attributes is never content, so pages render exactly
  as before. Screenshots of 7 pages in 4 styles at phone and desktop width
  were identical before and after, and the Nu HTML Checker passes.

## Consequences

- About 520 bytes more per page before compression, about 45 after gzip
  (about 2%).
- The harness checks the layout on every built page
  (`each_attribute_on_its_own_line`). Other harness checks read pages
  through `support::read`, which joins each tag back onto one line, so they
  keep matching tags like `<link rel="canonical"` as plain strings.
- A link in the middle of a paragraph now spans several lines in the
  source. It still renders inline with no added space, because the break
  is inside the tag, never in the text.

## Alternatives considered

- **Split only long tags (over about 100 columns).** Fewer lines, but
  whether a tag splits would depend on its values, so the same tag reads
  differently from page to page.
- **Put each inline element and its text on their own lines.** Whitespace
  inside `<a>`, `<time>` or `<span>` is content: it would add visible,
  underlined spaces to links.
- **Put the closing `>` on its own line.** Common in JSX, but the newline
  before the content would be text inside the element, with the same
  problem as above.
