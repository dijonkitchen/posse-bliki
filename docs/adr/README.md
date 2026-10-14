# Architecture Decision Records

Each ADR captures one decision: context, options considered, the choice made,
and consequences. ADRs are **append-only** — to change a decision, write a new
ADR that supersedes the old one. Don't edit history.

| # | Title | Status |
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
| [0012](0012-nu-html-checker-in-ci.md) | Nu HTML Checker in CI | Accepted |

## Format

Short. One page each. The four sections are: **Context**, **Decision**,
**Consequences**, **Alternatives considered**. Date and status at the top.
