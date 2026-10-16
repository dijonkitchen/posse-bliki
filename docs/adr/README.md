# Architecture Decision Records

Each ADR captures one decision: context, options considered, the choice made,
and consequences. ADRs are **append-only** — to change a decision, write a new
ADR that supersedes the old one. Don't edit history.

| Date | ADR | Status |
|---|---|---|
| 2026-05-10 | [IndieWeb standards in scope](2026-05-10-indieweb-standards-supported.md) | Accepted |
| 2026-05-10 | [No Quartz, no vendored SSG](2026-05-10-no-quartz-no-vendored-ssg.md) | Accepted |
| 2026-05-10 | [Obsidian as editor, no required plugins](2026-05-10-obsidian-as-editor-no-required-plugins.md) | Accepted |
| 2026-05-10 | [Spec + harness as the durable artifact, not the build code](2026-05-10-spec-and-harness-over-implementation.md) | Accepted |
| 2026-05-10 | [uv for Python toolchain](2026-05-10-uv-for-python-toolchain.md) | Superseded by [2026-10-11-rust-only-toolchain](2026-10-11-rust-only-toolchain.md) |
| 2026-10-06 | [Rust build with zero dependencies](2026-10-06-rust-build-zero-dependencies.md) | Accepted |
| 2026-10-11 | [a11y design system, self-hosted fonts](2026-10-11-a11y-design-system-self-hosted-fonts.md) | Accepted |
| 2026-10-11 | [Holocene years on the page](2026-10-11-holocene-years-on-the-page.md) | Accepted |
| 2026-10-11 | [Nu HTML Checker in CI](2026-10-11-nu-html-checker-in-ci.md) | Accepted |
| 2026-10-11 | [Rust-only toolchain: harness, tables and a pinned compiler](2026-10-11-rust-only-toolchain.md) | Accepted |
| 2026-10-11 | [Webmentions at deploy time](2026-10-11-webmentions-at-deploy-time.md) | Accepted |
| 2026-10-14 | [Mobile first, readable output, a Lighthouse budget](2026-10-14-mobile-first-readable-output-lighthouse.md) | Accepted |
| 2026-10-14 | [Site and author naming](2026-10-14-site-and-author-naming.md) | Accepted; shown author name superseded by [2026-10-16-site-title-heads-home](2026-10-16-site-title-heads-home.md) |
| 2026-10-14 | [Supply chain: SHA-pinned actions, least-privilege tokens](2026-10-14-supply-chain-pinned-actions.md) | Accepted |
| 2026-10-14 | [www as the canonical host](2026-10-14-www-as-canonical-host.md) | Accepted |
| 2026-10-16 | [Every ADR named by date](2026-10-16-date-named-adrs.md) | Accepted |
| 2026-10-16 | [Keep the old Substack and Jekyll feed URLs](2026-10-16-old-feed-urls.md) | Accepted |
| 2026-10-16 | [Redirect the old /all page to the notes index](2026-10-16-old-all-page.md) | Accepted |
| 2026-10-16 | [Relative Markdown links between notes](2026-10-16-relative-markdown-links.md) | Accepted |
| 2026-10-16 | [The site title heads the home page](2026-10-16-site-title-heads-home.md) | Accepted |

## Naming

New ADRs are named by date, `YYYY-MM-DD-short-slug.md`, so parallel branches
don't race for the next number. If two land on the same day, the slug keeps
them apart. The date is the day the decision was accepted. The ADRs that
were once numbered (0001–0016) were renamed to this form; see
[ADR 2026-10-16-date-named-adrs](2026-10-16-date-named-adrs.md) for the
mapping. Refer to an ADR by its file name stem,
e.g. `ADR 2026-10-14-www-as-canonical-host`.

## Format

Short. One page each. The four sections are: **Context**, **Decision**,
**Consequences**, **Alternatives considered**. Date and status at the top.
