# 2026-10-29 — Block content on its own lines

- Date: 2026-10-29
- Status: Accepted
- Decided by: JC, asking after one attribute per line (ADR
  2026-10-28-one-attribute-per-line) "Shouldn't content of tag be its own
  line too?", then choosing "block tags only" over every tag.

## Context

With attributes on their own lines, a list item still read as one run:
`<li class="h-entry"><a` … `wonder</a></li>`. The start tag, the content
and the end tag shared lines, so the nesting the indenting pass shows
stopped at the first inline element.

Whitespace means different things in different places. In an inline
element (`<a>`, `<span>`, `<time>`, `<em>`) it is text: a newline before
"wonder" inside `<a>` would render as a space, and an underlined link would
grow an underlined gap. In a block element the browser drops whitespace at
the start and end of each line, so a newline after `<li>` or before `</li>`
renders as nothing.

## Decision

- For block elements, the build (`build/indent.rs`) puts the content on its
  own lines, one level deeper than the start tag, and gives the end tag a
  line of its own:

  ```html
  <li class="h-entry">
    <a
      class="u-url p-name"
      href="/notes/wonder/">wonder</a>
  </li>
  ```

- Block means the list in `indent::BLOCK`: `li`, `p`, `div`, `ul`, `ol`,
  `dl`, `dt`, `dd`, `blockquote`, the sectioning elements (`header`,
  `footer`, `main`, `nav`, `section`, `article`, `aside`), `figure`,
  `figcaption`, `form`, `fieldset`, `address` and the table elements.
  An empty block (`<li></li>`) stays on one line.
- **Headings stay as they are.** Several styles draw on them with
  `::before` and `::after` (Terminal's prompt and blinking cursor, Mid
  Century's pill), and that generated content sits on the same line as the
  text. The space next to it is no longer at the start or end of the line,
  so it showed: a test run moved Terminal's cursor and Mid Century's
  heading text by one space.
- Inline elements keep their text against the tag, as before.

## Consequences

- Screenshots of 8 pages in all 20 styles at phone and desktop width (320
  pairs), plus 4 more pages with lists, quotes and tables in every style,
  matched before and after. The only differences were in Steel and Doctor
  Who: a pixel or two of antialiasing and an animated lamp glow, which also
  differ when the same page is shot twice.
- The harness checks the layout on every built page
  (`block_content_on_its_own_lines`). `support::read` joins block content
  back onto its tags, as it joins attributes, so other checks still match
  strings like `<p class="site-tagline">…</p>`.
- A style that adds `::before` or `::after` content to one of these block
  elements, or makes one `display: inline`, would show the new spaces.
  Style a heading, or wrap the text in an inline element, instead. The
  screenshot comparison in the PR is the way to check.
- More lines per page, but the bytes added are newlines and indentation,
  which gzip compresses to almost nothing.

## Alternatives considered

- **Every tag's content on its own line.** It would add visible spaces
  inside links and between inline elements (JC chose against it).
- **Include headings and change the styles** so their `::before` and
  `::after` are positioned rather than inline. That is a redesign of five
  styles for a view-source nicety, and every new style would have to know
  about it.
