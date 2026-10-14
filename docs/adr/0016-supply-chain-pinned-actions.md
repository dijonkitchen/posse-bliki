# 0016 — Supply chain: SHA-pinned actions, least-privilege tokens

- Date: 2026-10-14
- Status: Proposed (accepted when JC merges this PR)
- Raised by: JC, asking how supply chain safety could be improved.

## Context

The build has no crates (ADR 0007) and a pinned compiler (ADR 0010), so
almost all third-party code that touches the site now runs in GitHub
Actions: `checkout`, `configure-pages`, `upload-artifact`,
`upload-pages-artifact`, `deploy-pages` and `download-artifact`.

They were referenced by tag (`actions/checkout@v4`). A tag can be moved to
new code at any time, by the maintainer or by whoever steals their
credentials. That happened to `tj-actions/changed-files` in March 2025:
every tag was repointed to code that dumped CI secrets into public logs.
A mutable tag runs whatever it points to on the day of the run.

The deploy workflow also gave every job `pages: write` and
`id-token: write`, including the build job that runs repository code and
the `make ci` gate. `actions/checkout` also leaves the token in
`.git/config` by default, where any later step can read it.

## Decision

- Pin every third-party action to a full 40-character commit SHA, with the
  release in a trailing comment: `uses: actions/checkout@<sha> # v4.4.0`.
  A commit SHA cannot be changed after the fact.
- Dependabot (GitHub's own bot, no third-party app) proposes SHA bumps
  monthly, grouped into one PR, after a release has aged seven days.
- Permissions are set per job, least first. The workflow default is
  `contents: read`; the build job adds `pages: read`; only the deploy job,
  which runs no repository code, gets `pages: write` and `id-token: write`.
- `actions/checkout` runs with `persist-credentials: false`. Nothing
  after checkout pushes.
- Cargo runs with `--locked`, so a stray crate or lockfile change fails
  CI instead of being resolved silently.
- `tests/harness/supply_chain.rs` fails if an action is not SHA-pinned or
  `Cargo.lock` lists any package but this one.

## Consequences

- An action only changes when a reviewed PR changes its SHA.
- One more PR a month at most, from Dependabot.
- New workflow steps must be pinned too; the harness says which line.
  Open PRs that add `@v4` references (the Nu checker and Lighthouse jobs)
  need their SHAs pinned before or after merging.
- A SHA pin does not cover what an action downloads at run time. The
  Rust toolchain comes from rustup, which checks each download against
  the signed release manifest; `make tables` sources are reviewed through
  the regenerated diff.

## Alternatives considered

- **Keep tags, trust GitHub's own actions.** All six actions are from the
  `actions/` org, which is lower risk, but the rule is simpler and
  testable if it has no exceptions.
- **Renovate.** The old Jekyll site uses it. Dependabot is built into
  GitHub and needs no app install or extra permissions.
- **Write the steps by hand (no actions).** Checkout and Pages deploy can
  be done with `git` and `curl`, but that is more code to own than a
  pinned, well-reviewed action.
- **Harden-Runner or egress allowlisting.** Worth it for repos with
  secrets in CI; this one only has `WEBMENTION_IO_TOKEN`. Revisit if more
  secrets arrive.
