# 2026-10-28 — Modern JavaScript in the style picker

- Date: 2026-10-28
- Status: Accepted
- Decided by: JC, asking whether `/style-picker.js` should use more modern
  JS with `const fn-name = () => ...`, then choosing "const arrows
  throughout", and "use latest best practices, especially if they are
  compatible cross browser for many users".

## Context

`/style-picker.js` is the site's only script (ADR 2026-10-18-style-picker)
and ships unminified so readers can read and remix it from view-source
(ADR 2026-10-14-mobile-first-readable-output-lighthouse). It was written in
ES5 style: `var`, `function` declarations and index loops. Yet it already
relied on `URL`, `URLSearchParams` and `blocking="render"`, and the picker
itself needs CSS `:has()` (Safari 15.4, Chrome 105, Firefox 121). Every
browser that can use the picker has had `const`, `let`, arrow functions,
spread and template literals since about 2016, and optional `catch`
bindings since 2018-2019.

## Decision

- `const` for every binding that never changes, `let` for the one that does
  (the saved style). No `var`.
- Helpers are `const name = () => ...` arrows, and so are the callbacks.
- The menu's values are read once into an array; `pick` uses `includes` and
  April Fools' Day uses `filter` instead of index loops. Already-loaded
  stylesheets are a `Set`.
- `"use strict"` at the top of the wrapper, so mistakes such as assigning
  an undeclared name throw instead of quietly making a global.
- Only syntax every browser with `:has()` supports. Newer features that
  would narrow support for no gain here (top-level `await`, private
  fields, `Array.prototype.at`-era APIs) aren't needed and aren't used.
- The wrapper stays an immediately invoked function (now an arrow), not
  `<script type="module">`: modules are deferred, so a remembered style
  would flash in after the page is drawn.
- Behavior is unchanged.

## Consequences

- The script is shorter and says more: `const` tells the reader a name is
  never reassigned.
- Arrows aren't hoisted the way `function` declarations are: calling a
  helper above its `const` line throws. Every helper is defined before it's
  used, and the harness test `style_picker_uses_modern_syntax` checks that,
  plus that no `var` or `function` comes back.
- Arrows have no own `this` or `arguments`; the script uses neither. Stack
  traces still show helper names, inferred from the `const`.
- A browser without ES2015 now gets a syntax error instead of a working
  picker. Such a browser has no `:has()` either, so it already showed only
  the house style; with JS off or broken the picker still restyles the
  current page wherever `:has()` works.

## Alternatives considered

- **Keep ES5.** Works everywhere, but buys nothing: no browser that runs
  the picker needs it.
- **`const`/`let` but keep `function` declarations.** Keeps hoisting, so
  order wouldn't matter. JC preferred arrows throughout; the order is
  natural here and tested.
