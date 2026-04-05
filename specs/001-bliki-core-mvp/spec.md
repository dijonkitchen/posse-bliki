# Feature Specification: Foundational bliki core (static site from Markdown)

**Feature Branch**: `001-bliki-core-mvp`  
**Created**: 2026-04-05  
**Status**: Draft  
**Input**: User description: "Foundational posse-bliki: authors create and edit Markdown pages with titles and slug URLs; browse a sorted page list; full-text search across page content; show last-modified metadata. Target a single author or small trusted group. Public comments and social features are out of scope for this slice."

## Clarifications

### Session 2026-04-05

- Q: What is the authoring and deployment model for posse-bliki MVP? → A: Single operator with repo access; add/edit Markdown source files directly in the codebase; **no authentication** in the product; **static site generation** for publishing (HTML/assets suitable for static hosting).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Publish a new page (Priority: P1)

A single operator adds a new Markdown source file (with agreed front matter or file naming), runs the site
generator, deploys or opens the built output, and sees the page at a stable, human-readable URL with
rendered content.

**Why this priority**: Without generated readable pages from Markdown sources, there is no bliki; this is
the smallest shippable slice.

**Independent Test**: Add one Markdown file, build, open the corresponding URL in the static output, and
confirm title and body match without using list or search features.

**Acceptance Scenarios**:

1. **Given** no page exists yet, **When** the operator adds a Markdown source file per documented layout
  rules and runs the build, **Then** the static output contains a page at a documented slug URL with
   rendered content.
2. **Given** a page exists in the build output, **When** a reader opens that URL (e.g., from static
  hosting or local preview), **Then** they see the title and rendered body.

---

### User Story 2 - Revise an existing page (Priority: P2)

The operator edits Markdown source in the repository, rebuilds, and sees updated content and
last-modified presentation in the output.

**Why this priority**: Iteration on writing is core; edits happen at the file level for this model.

**Independent Test**: Change body text in a source file, rebuild, reload the page URL, and confirm new text
and updated last-modified indication without using search or list features.

**Acceptance Scenarios**:

1. **Given** a page exists, **When** the operator changes title or body in source and rebuilds, **Then** the
  static output shows the new content at the same canonical URL (unless slug rules intentionally change,
   which must be documented).
2. **Given** a page exists, **When** sources are updated, **Then** last-modified metadata in the output
  reflects the change (from file timestamps or front matter—documented convention).

---

### User Story 3 - Browse and search (Priority: P3)

A reader (or the operator verifying the site) opens an index or list of pages in the static output, uses
search to find pages whose title or body matches terms, and opens a result.

**Why this priority**: Discovery matters once more than a few pages exist.

**Independent Test**: With several pages built, locate a known page via the generated search and via the
list without editing sources in that session.

**Acceptance Scenarios**:

1. **Given** multiple pages exist in the build, **When** the user opens the list/index page, **Then**
  entries appear in a consistent documented order and link to static page URLs.
2. **Given** multiple pages exist, **When** the user searches for a term present in a page’s title or body,
  **Then** that page appears in search results and can be opened, **without** requiring a server-side
   application for search (e.g., client-side index loaded with the site or equivalent static-friendly
   approach documented in planning).

---

### Edge Cases

- **Slug collision**: Two source files must not map to the same output URL without an explicit
build-time error or deterministic resolution rule that is visible to the operator.
- **Empty or minimal content**: Missing title or empty body must be handled with clear validation or
defaults at build time (no silent empty publish unless explicitly allowed).
- **Concurrent edits**: Multiple editors are out of scope; the single operator relies on normal VCS
workflows. If merge conflicts occur in Git, resolution happens outside the generator; the build MUST
fail or warn clearly if sources are invalid after merge.
- **Large pages**: Very long Markdown must still build and render without corrupting content; if limits
exist, the build reports them clearly.
- **No runtime auth**: The published static site MUST NOT expose create/edit in the browser that mutates
content without going through source files (no hidden admin API in this slice).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST define how Markdown sources map to output URLs (slugs), with uniqueness enforced
at build time across all pages.
- **FR-002**: System MUST generate a **static site** (HTML and supporting assets) deployable to static
hosting without a dynamic application server for reads.
- **FR-003**: Build MUST render Markdown to readable HTML for each page in the output.
- **FR-004**: Operator MUST be able to add and edit Markdown in the repository; rebuilding MUST update the
static output to reflect those sources.
- **FR-005**: Generated site MUST include a browsable list or index of pages with a consistent sort order
documented for operators and readers.
- **FR-006**: Generated site MUST provide search across page titles and bodies that works in a static
hosting context (e.g., build-generated index with client-side search, or another approach that does not
require server-side query execution at read time); exact mechanism is chosen in planning but MUST meet
this constraint.
- **FR-007**: Product MUST NOT require authentication for reading or for the generator’s published output;
editing is **only** via source files in the repo (no in-browser authenticated editor in this slice).
- **FR-008**: System MUST NOT include public comments, reactions, or social feeds in this slice.

### Key Entities *(include if feature involves data)*

- **Page (source)**: Markdown file plus metadata (front matter and/or path convention) used by the generator
to produce one static page.
- **Built page**: HTML (and assets) emitted for a single URL, derived from exactly one Page (source) unless
documentation defines aliases (not required in MVP).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new operator can add a first Markdown page, run a successful build, and open the matching
URL in the output in under 10 minutes using only project documentation (no account setup).
- **SC-002**: With 20 built pages averaging 500 words each, search from the static site finds a distinctive
term within 10 seconds from user action to visible results on a broadband connection.
- **SC-003**: 100% of source edits intended for publish appear in the rebuilt static output at the expected
URL in acceptance testing (no silent drops).
- **SC-004**: A reader can complete “find and open a page from the list” without assistance for a labeled
task in moderated usability-style testing (pass/fail per session).

## Assumptions

- **Single operator** with repository access; trust and access control are organizational (Git hosting),
not application-level auth.
- Public comments and social features are explicitly excluded from this slice.
- Markdown flavor and rendering rules follow one documented set in planning; default is common
GitHub-flavored-style unless constrained.
- Deployment target is **static hosting** (object storage, CDN, or simple file server); dynamic server
features are out of scope for MVP unless added in a later spec.
- Mobile-native apps are out of scope; reading experience is the generated site in a browser.