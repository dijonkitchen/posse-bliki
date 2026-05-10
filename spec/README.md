# Specs

Outcome contracts for this site. **The build is correct iff every spec here
is satisfied** — and every spec has a corresponding test in `../tests/`.

Read order:

1. [content-schema.md](content-schema.md) — what notes look like (input contract)
2. [url-policy.md](url-policy.md) — where notes end up (URL contract)
3. [output-contract.md](output-contract.md) — what HTML pages must contain
4. [feeds.md](feeds.md) — RSS, JSON Feed, sitemap
5. [indieweb.md](indieweb.md) — which IndieWeb standards are supported
6. [build-invariants.md](build-invariants.md) — hermetic, idempotent, deterministic

## How to change a spec

1. Edit the spec.
2. Add or update the failing test in `tests/`.
3. Make the build pass.
4. If it's a notable architectural shift, also write an ADR in `docs/adr/`.

The spec is the source of truth. The build is regenerable.
