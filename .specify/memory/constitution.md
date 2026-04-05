

# posse-bliki Constitution

## Core Principles

### I. Content-first & portable

Readable Markdown (or equivalent plain-text markup) is the canonical source for authored pages. Published
pages MUST be representable as stable, human-meaningful URLs (slugs) and remain export-friendly (no
vendor-locked binary-only formats for primary content). Changing a title MUST NOT silently break inbound
links without an explicit migration or redirect strategy when URLs are user-facing.

**Rationale**: A bliki’s value is durable, shareable writing; portability reduces lock-in and supports
archival and migration.

### II. Simplicity over cleverness

Prefer a small, understandable stack and explicit modules over frameworks that obscure data flow. New
dependencies MUST solve a concrete problem that cannot be met with existing tools at proportionate cost.
YAGNI applies: do not build generalized platforms before a second concrete use case exists.

**Rationale**: Small teams ship and maintain simple systems longer than clever ones.

### III. Test-first for domain logic (NON-NEGOTIABLE)

For rules that can be isolated—Markdown processing, slug generation, search indexing behavior, permission
checks—write automated tests before or alongside implementation (red-green-refactor). Critical user
journeys (e.g., create → view → edit → search) MUST have at least one automated integration or end-to-end
check where feasible; if not feasible, document the gap and the manual verification steps.

**Rationale**: Regressions in text handling and URLs erode trust quickly; tests encode the real contract.

### IV. Trust boundary clarity

Authentication and authorization models MUST be explicit in specs and plans: who can read, create, edit,
and delete content. Secrets and credentials MUST NOT be committed; configuration MUST use environment or
secure secret stores. User-visible errors MUST avoid leaking internal implementation details.

**Rationale**: Wikis and blogs often handle semi-private material; ambiguity causes data exposure incidents.

### V. Spec-driven delivery

Non-trivial work follows: constitution compliance → feature specification → technical plan → tasks →
implementation. Deviations (e.g., hotfixes) MUST be reconciled afterward by updating artifacts or recording
a time-boxed exception with owner.

**Rationale**: Aligns tooling (Spec Kit) with accountable, reviewable change.

## Content & quality standards

- **Accessibility**: Primary reading and editing flows SHOULD meet a reasonable baseline (semantic structure,
keyboard operability for core actions) unless a feature explicitly defers accessibility with a documented
follow-up.
- **Performance (read path)**: Typical page read and search results SHOULD feel immediate on a broadband
connection for expected content sizes; specific numeric targets belong in feature specs or plans once
scale is known.
- **Writing experience**: Editing SHOULD preserve intent (whitespace and common Markdown constructs) unless
a normalization rule is documented and tested.

## Development workflow

- **Branches**: Feature work uses numbered feature branches (e.g., `001-feature-name`) consistent with
repository conventions and Spec Kit scripts.
- **Review**: Changes that touch permissions, persistence, or URL behavior SHOULD be reviewed before merge
when more than one contributor exists.
- **Definition of done**: Merged work includes tests required by Principle III for the change, updated docs
or specs when behavior is user-visible, and no known open security regressions for the touched surface.

## Governance

This constitution supersedes ad hoc coding preferences when they conflict. Amendments require updating
this document, bumping **Version** per semantic versioning (MAJOR: incompatible governance; MINOR: new
principle or materially expanded rule; PATCH: clarifications only), and setting **Last Amended** to the
change date. Pull requests SHOULD state constitution impact when non-obvious.

**Version**: 1.0.0 | **Ratified**: 2026-04-05 | **Last Amended**: 2026-04-05