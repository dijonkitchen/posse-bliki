# 2026-11-05 — Nix is still not worth it

- Date: 2026-11-05
- Status: Accepted (JC chose "Skip Nix, add Lighthouse to Dependabot", 2026-11-05)
- Decided by: JC, asking "worth using nix to manage all deps?" with a
  link to a CI run on #72; Claude recommended against it.

## Context

[ADR 2026-10-11-rust-only-toolchain](2026-10-11-rust-only-toolchain.md)
chose `rust-toolchain.toml` over Nix because Rust was the only tool the
repo needed pinned. CI has since added more tools, so the question came
back.

The run JC linked was GitHub's own code scanning AI review of #72. It
passed with no findings. GitHub sets up that job itself, so nothing in
this repo, Nix included, changes what it runs.

Here is what each tool runs at today:

| Tool | Where it comes from | Pinned |
| --- | --- | --- |
| Rust, rustfmt, clippy | rustup, from `rust-toolchain.toml` | exact; Dependabot bumps it |
| Crates | none | nothing to pin |
| GitHub Actions | commit SHAs | exact; Dependabot bumps them |
| Nu HTML Checker | Makefile `VNU_VERSION` + SHA-512 | exact; bumped by hand |
| Lighthouse | `scripts/lighthouse/package-lock.json` | exact; Dependabot bumps it ([ADR 2026-11-05-dependabot-lighthouse-updates](2026-11-05-dependabot-lighthouse-updates.md)) |
| Node | `actions/setup-node`, `22` | major only |
| Java | `actions/setup-java`, Temurin `21` | major only |
| Chrome, jq, curl, make | the `ubuntu-24.04` runner image | whatever the image ships |

None of the CI failures so far came from a tool version changing: Chrome
starting too slowly for Lighthouse ([ADR
2026-10-29-lighthouse-chrome-retry](2026-10-29-lighthouse-chrome-retry.md)),
the Diablo style shifting layout ([ADR
2026-10-30-style-before-first-paint](2026-10-30-style-before-first-paint.md)),
a Pages deploy stuck in progress, and the W3C feed validator rate-limiting
CI. Pinning every tool with Nix would not have prevented any of them.

One thing has changed since the October ADR. Dependabot can now update a
`flake.lock` (since April 2026):
https://github.blog/changelog/2026-04-07-dependabot-version-updates-now-support-the-nix-ecosystem
So Nix would no longer need Renovate or a separate updater.

## Decision

Don't adopt Nix. Keep pinning each tool where it lives, as in the table,
and let Dependabot propose the bumps.

## Consequences

- Chrome, Node and Java can still change under CI when the runner image
  or a setup action updates. If one of those ever breaks a check, that's
  the time to pin it, starting with the tool that broke rather than
  everything.
- Setting up a machine stays `make setup` (rustup) for the build and
  tests. Java, Node and Chrome are needed only to run `make validate` or
  `make lighthouse` locally.
- Cloud sessions keep working without installing anything beyond rustup.

## Alternatives considered

- **Nix flake for CI and a dev shell.** Pins every tool, Chrome included,
  and gives one command to set up a machine. Costs:
  - every job that needs tools would install Nix and download Chromium, a
    JDK, Node and Rust. That's test, html, the page list and one
    Lighthouse job per page (up to 16), so up to 19 jobs. Without a binary
    cache, that's likely slower than today's roughly two-minute CI.
  - Chromium in nixpkgs builds only for Linux, so `make lighthouse` on a
    Mac still wouldn't match CI.
  - Dependabot bumps the whole `nixpkgs` input in one PR, so Chrome, Java
    and Node would move together, and a break is harder to trace to one
    tool.
  - It's one more system for contributors (and Claude's cloud sessions)
    to install and learn, which was the original reason against it.
- **Nix dev shell only, CI unchanged.** Local and CI versions could then
  drift apart, which defeats the point.
- **Pin Node and Java to exact versions in the workflows.** Dependabot
  doesn't bump `node-version` or `java-version`, so they would age until
  someone remembered, and the major-only pins already get patch and
  security fixes.
