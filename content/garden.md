---
title: Style garden
summary: One page to try every style in the picker on, after the CSS Zen Garden. Each style is one stylesheet you can read and remix.
---

This page is a small homage to the [CSS Zen Garden](https://csszengarden.com),
where Dave Shea asked designers to restyle one fixed HTML page using only CSS.
Pick a style from the menu at the top of any page and this one shows off what
it does.

## How it works

The house style is `/style.css`. Each extra style is a short file in
`/themes/` that changes the colour tokens, the fonts and a few flourishes,
and nothing else. The menu switches between them with CSS alone, so it works
with JavaScript off. A few lines of JavaScript load only the style you pick,
so the others cost nothing, and remember it as you move between pages;
without it, every style loads and the next page starts in the house style
again. View the source, copy a file, and make your own.

Auto follows your device's light or dark setting. Light and Dark pin one.
If you've never picked a style, the bridge lights up for Star Trek Day on
8 September, Diablo haunts the site for Halloween week, the TARDIS lands
on 23 November for Doctor Who's birthday, Holiday decks the halls in
December, and on April Fools' Day every page comes up in a style drawn at
random from the whole menu.

The CSS Zen Garden styles are new stylesheets written in tribute to famous
designs from the Garden, not copies: no images or fonts, every shape drawn
with CSS, and the text kept at AAA contrast. The originals belong to their
designers, and are well worth a visit.

## A bit of everything

Every style has to cope with the same things a note does:

- a list, with **bold** and *italic* text
- `inline code` and [a link](about.md)
- a quotation, a table and a block of code

> Constraint breeds creativity.

| Style | Mood |
|---|---|
| Auto, Light, Dark | calm and readable, the house style |
| Terminal | green phosphor at 2 a.m. |
| Broadsheet | after the Financial Times: salmon paper and serif headlines |
| Solarized Light | Ethan Schoonover's Solarized, in JetBrains Mono |
| Editor Dark | VS Code and JetBrains in the dark, in JetBrains Mono |
| Tranquille | Dave Shea's first Zen Garden design: mist and moss |
| Mid Century | after Andrew Lohman's Mid Century Modern: 1958 living room |
| Steel | after Steffen Knoeller's Steel: riveted plate |
| Apothecary | after Trent Walton's Apothecary: an old chemist's label |
| Diablo | Tristram, 1996; turns itself on for Halloween week |
| Doctor Who | the TARDIS in the console room; turns itself on for the show's birthday |
| Star Trek | the ship's computer, with space outside; turns itself on for Star Trek Day |
| Holiday | pine, cranberry and snow; turns itself on in December |
| GeoCities | a 1997 home page, under construction |
| Angelfire | neon in space, with flames |
| Xanga | a 2004 blog, with dotted borders |
| Jekyll | this site's first home, Jekyll's 2015 default theme |
| Facebook | the 2007 profile page and its blue bar |

```css
:root:has(#style [value="terminal"]:checked) {
  --text-1: #7dff9b;
  --surface-page: #0b100c;
}
```
