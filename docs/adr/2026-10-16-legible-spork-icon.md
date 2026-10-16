# 2026-10-16 — A tab icon closer to the spork picture, legible at a distance

- Date: 2026-10-16
- Status: Accepted
- Raised by: JC, who asked for the spork SVG to look more like the Substack
  picture, to be easy to understand at a distance, and to have a rounder
  tip like a real spork. Accepted when JC merges the PR that adds this ADR.

## Context

The first tab icon (ADR 2026-10-14-mobile-first-readable-output-lighthouse)
was a generic fork: a squat, nearly square head with stubby slots on a
straight stick, tilted 40°. The Substack picture is a line drawing of a
spork with a long oval bowl, three tines at the tip, a narrow neck and a
flat handle that widens to a rounded end, at about 33°. Its hairlines and
shading vanish below about 64px, so the drawing itself can't be the tab
icon.

## Decision

Redraw `icon.svg` as one solid silhouette that follows the picture's
outline and slant: a bowl with a round spoon tip, like a real spork,
rounding into a narrow neck, three short tines cut into the tip by two
slots wide enough to stay open at 16px, and a handle that widens toward
its end. No outlines, hairlines or shading. Ink `#1e232c` on light tabs
and paper `#fbfaf7` on dark ones, both over 15:1 against their
background. `favicon.ico` is rendered from the same SVG at 32px. The
180px, 192px and 512px icons stay the picture itself, where its detail
survives. Three tines, as in the picture: a four-tine version was tried,
and its slots blur together at 32px.

## Consequences

- The icon reads as a spork, not a fork, and keeps its shape at a glance
  and at a distance.
- The small icons and the large ones now share the picture's proportions.

## Alternatives considered

- **Trace the drawing's outlines as strokes**: closest to the picture, but
  thin lines blur to grey at 16px, the opposite of legible.
- **Keep the stylised fork**: legible, but not recognisably the picture.
