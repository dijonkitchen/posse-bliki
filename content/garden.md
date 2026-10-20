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
If you've never picked a style, Diablo haunts the site for Halloween week
and Holiday decks the halls in December.

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
| Broadsheet | the morning paper |
| Blueprint | drafting table |
| Tranquille | Dave Shea's first Zen Garden design: mist and moss |
| Mid Century | after Andrew Lohman's Mid Century Modern: 1958 living room |
| Steel | after Steffen Knoeller's Steel: riveted plate |
| Apothecary | after Trent Walton's Apothecary: an old chemist's label |
| Under the Sea | after Eric Stoltz's Under the Sea!: deep water |
| Garments | after Dan Mall's Garments: denim and top-stitching |
| Diablo | Tristram, 1996; turns itself on for Halloween week |
| Holiday | pine, cranberry and snow; turns itself on in December |
| GeoCities | a 1997 home page, under construction |
| Angelfire | neon in space, with flames |
| Xanga | a 2004 blog, with dotted borders |
| Jekyll | this site's first home, Jekyll's 2015 default theme |

```css
:root:has(#style [value="terminal"]:checked) {
  --text-1: #7dff9b;
  --surface-page: #0b100c;
}
```
