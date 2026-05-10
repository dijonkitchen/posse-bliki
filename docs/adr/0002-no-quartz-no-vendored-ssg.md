# 0002 — No Quartz, no vendored SSG

- Date: 2026-05-10
- Status: Accepted

## Context

Quartz 4 is the canonical Obsidian-vault → static-site tool. An earlier draft
of this repo used it. It gives wikilinks, backlinks, graph view, search, and
popovers out of the box — significant ergonomic wins.

It also brings ~50k LoC of TypeScript, ~200 transitive npm packages, and
periodic major-version rewrites (v3 → v4). For a 10+ year horizon, and
especially under agentic maintenance, that's a problematic surface.

## Decision

Do not use Quartz, Hugo, 11ty, Astro, or any other off-the-shelf SSG. Write
the smallest build script that satisfies `spec/` and passes `tests/`.

## Consequences

- Repo is small enough that an agent can hold the whole build in context.
- Behaviour is fully specified, not inherited from a third party.
- Upgrading is bounded by our own deps, not by an SSG's release cadence.
- Cost: we lose Quartz's polished features (graph view, popovers, fancy
  search). For a personal bliki this is acceptable — ASCII-readable
  markdown + working wikilinks + backlinks is the core need.

## Alternatives considered

- **Quartz, fetched at build time.** Smaller repo than vendoring, but the
  Node + npm dependency surface is unchanged, and behaviour is still
  unspecified.
- **Hugo.** Single Go binary is genuinely durable. Templating language is
  opaque to current agents and Obsidian wikilink support requires
  third-party shortcodes/themes. If the personal-build hypothesis fails,
  Hugo is the recommended migration target — see ADR 0001 on the harness
  enabling such a move.
- **11ty.** Better Obsidian story than Hugo via plugins. Same npm-surface
  concerns as Quartz, smaller scale.
