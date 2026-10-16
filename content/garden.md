---
title: Style garden
summary: The same page in different clothes, after the CSS Zen Garden. Every style is one stylesheet you can read and remix.
---

This page is a small homage to the [CSS Zen Garden](https://csszengarden.com),
where Dave Shea asked designers to restyle one fixed HTML page using only CSS.
Each style here is the same markup with one extra stylesheet on top of the
house one. Pick one from the list above.

## How it works

The house style is `/style.css`. Each garden style is a short
file in `/themes/` that changes the colour tokens, the
fonts and a few flourishes, and nothing else. View the source, copy a file,
and make your own.

There is no JavaScript on this site and nothing to remember you by, so a style
you pick here stays on this page. The rest of the site follows your device's
light or dark setting.

## A bit of everything

Every style has to cope with the same things a note does:

- a list, with **bold** and *italic* text
- `inline code` and [a link](about.md)
- a quotation, a table and a block of code

> Constraint breeds creativity.

| Style | Mood |
|---|---|
| House | calm, readable, follows your device |
| Terminal | green phosphor at 2 a.m. |
| Broadsheet | the morning paper |
| Blueprint | drafting table |

```css
:root {
  --text-1: #7dff9b;
  --surface-page: #0b100c;
}
```
