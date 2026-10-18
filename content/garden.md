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
with JavaScript off. A few lines of JavaScript remember your pick as you move
between pages; without it, the next page starts in the house style again.
View the source, copy a file, and make your own.

Auto follows your device's light or dark setting. Light and Dark pin one.

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

```css
:root:has(#style [value="terminal"]:checked) {
  --text-1: #7dff9b;
  --surface-page: #0b100c;
}
```
