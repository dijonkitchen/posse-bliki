# Architecture Decision Records

Each ADR captures one decision: context, options considered, the choice made,
and consequences. ADRs are **append-only** — to change a decision, write a new
ADR that supersedes the old one. Don't edit history.

| ADR | Title | Status |
|---|---|---|
| [0001](0001-spec-and-harness-over-implementation.md) | Spec + harness as the durable artifact, not the build code | Accepted |
| [0002](0002-no-quartz-no-vendored-ssg.md) | No Quartz, no vendored SSG | Accepted |
| [0003](0003-uv-for-python-toolchain.md) | uv for Python toolchain | Superseded by 0010 |
| [0004](0004-indieweb-standards-supported.md) | IndieWeb standards in scope | Accepted |
| [0005](0005-obsidian-as-editor-no-required-plugins.md) | Obsidian as editor, no required plugins | Accepted |
| [0007](0007-rust-build-zero-dependencies.md) | Rust build with zero dependencies | Accepted |
| [0008](0008-webmentions-at-deploy-time.md) | Webmentions at deploy time | Accepted |
| [0009](0009-a11y-design-system-self-hosted-fonts.md) | a11y design system, self-hosted fonts | Accepted |
| [0010](0010-rust-only-toolchain.md) | Rust-only toolchain: harness, tables and a pinned compiler | Accepted |
| [0011](0011-holocene-years-on-the-page.md) | Holocene years on the page | Accepted |
| [0016](0016-supply-chain-pinned-actions.md) | Supply chain: SHA-pinned actions, least-privilege tokens | Accepted |
| [2026-10-11](2026-10-11-nu-html-checker-in-ci.md) | Nu HTML Checker in CI | Accepted |
| [2026-10-14](2026-10-14-site-and-author-naming.md) | Site and author naming | Accepted |
| [2026-10-14](2026-10-14-www-as-canonical-host.md) | www as the canonical host | Accepted |
| [2026-10-14](2026-10-14-mobile-first-readable-output-lighthouse.md) | Mobile first, readable output, a Lighthouse budget | Accepted |
| [2026-10-16](2026-10-16-old-feed-urls.md) | Keep the old Substack and Jekyll feed URLs | Accepted |
| [2026-10-16](2026-10-16-old-all-page.md) | Redirect the old /all page to the notes index | Accepted |

## Naming

New ADRs are named by date, `YYYY-MM-DD-short-slug.md`, so parallel branches
don't race for the next number. If two land on the same day, the slug keeps
them apart. Numbered ADRs already merged (0001–0016) keep their numbers:
they're append-only and linked from specs, code and commit history. Refer to an ADR by its file name stem,
e.g. `ADR 2026-10-14-www-as-canonical-host`.

## Format

Short. One page each. The four sections are: **Context**, **Decision**,
**Consequences**, **Alternatives considered**. Date and status at the top.
