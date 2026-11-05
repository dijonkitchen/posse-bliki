# 2026-10-22 — Broadsheet after the FT, two editor styles, and April Fools' Day from every style

- Date: 2026-10-22
- Status: Accepted
- Decided by: JC: "broadsheet should be like financial times. add a
  solarized light, and jetbrains/vscode default dark theme too (those can
  be in jetbrains mono or best mono typeface for coding)", then "make
  april fools one random".

## Context

Broadsheet (ADR 2026-10-18-style-picker) was a generic newspaper:
newsprint, Charter, a drop cap. JC wants it to read as the Financial
Times. JC also wants the two colour schemes people know from their code
editors, in a coding typeface. The site already self-hosts JetBrains Mono
as its code face (spec/design.md), so using it costs nothing.

Several of the originals' colours miss WCAG AAA 7:1 on their own
backgrounds, which every style here must meet (the harness checks): the
FT's teal `#0d7680` on its salmon is 4.8:1; Solarized's body grey
`#657b83` on base3 is 4.1:1 and its blue `#268bd2` 3.4:1; VS Code's
comment green `#6a9955` on `#1e1e1e` is 5.0:1.

On April Fools' Day, readers who never picked a style got a random style
from the five Throwbacks, from a list kept in the script.

## Decision

- **Broadsheet becomes the FT**, keeping its name and slug so saved picks
  still work: salmon paper `#fff1e5`, slate ink, Georgia headlines over
  Helvetica or Arial text, a serif masthead in capitals, section heads as
  small sans capitals under a short thick bar, teal links turning claret
  on hover, and a claret quote mark on pull quotes. No logo, no FT fonts
  (Financier and Metric are commercial).
- **Solarized Light**, after Ethan Schoonover's palette: base3 paper, base02
  text, blue links and magenta hover, headings in orange, green and cyan,
  all in JetBrains Mono.
- **Editor Dark**, after VS Code's Dark Modern and Dark+ and the JetBrains
  dark default, which share the look: charcoal panels, the title bar with
  the site name as the active tab, headings coloured like syntax
  (function yellow, type teal, variable blue), quotes as `//` comments,
  inline code in string orange, and a blue status bar for the footer, in
  JetBrains Mono. One style, not two, because the two editors' defaults
  differ only in small greys.
- **Colours that miss AAA are darkened or lightened along their own hue**
  until they pass, and each file says so. They stay recognisable.
- Both new styles go in a new **Editors** group after the Originals.
- **April Fools' Day draws from every style** in the menu, apart from
  Auto, Light and Dark, read from the menu itself, so a new style joins
  in without editing the script. Still drawn again on each page, still
  never saved.
- The rules of ADR 2026-10-20-zen-garden-styles hold: each sheet loads
  only when picked and is linked in `<noscript>` for readers without
  JavaScript.

## Consequences

- Seventeen styles. Pages are no heavier for readers with JavaScript;
  without it, two more small stylesheets load, as every style does.
- Readers who picked Broadsheet get a different look on their next visit.
- Solarized and Editor Dark are a little darker or lighter than the
  palettes their fans know.
- On 1 April a page can come up in any style, including the seasonal
  ones.

## Alternatives considered

- **Keep Broadsheet and add an FT style.** Two newspaper styles would be
  the kind of overlap ADR 2026-10-21-doctor-who-style removed.
- **Separate VS Code and JetBrains styles.** Near duplicates.
- **Solarized's exact colours.** Faithful, but under AAA, which every
  style meets.
- **Another coding font** (Fira Code, Cascadia Code, Iosevka). Each would
  be a new download; JetBrains Mono is already on the site and is the
  JetBrains default.
- **Keep April Fools' Day to the Throwbacks.** JC asked for it to be
  random.
