// Loads and remembers the style picked in the header, the only script on
// the site (ADR 2026-10-18-style-picker, ADR 2026-10-20-zen-garden-styles).
// Without it the picker still restyles the page you're on, through CSS
// :has() and the stylesheets in <noscript>; it just forgets.
// It runs where it sits, right after the picker, so a remembered style is
// in place before the page below it is drawn. Modern syntax, since every
// browser that has :has() has it; helpers are arrows, so each one must be
// defined above its first use (ADR 2026-10-28-modern-js-style-picker).
(() => {
  "use strict";
  const picker = document.getElementById("style");
  if (!picker) return;
  const house = document.querySelector('link[rel="stylesheet"][href$="style.css"]');
  const values = [...picker.options].map((option) => option.value);

  // A style's stylesheet, added the first time it's picked. Auto, Light and
  // Dark live in style.css. Render-blocking, so the page doesn't flash the
  // house style first. Only menu values ever reach here, and the URL is
  // resolved against the house sheet so it works under a sub-path.
  const loaded = new Set(["auto", "light", "dark"]);
  const load = (name) => {
    if (!house || loaded.has(name) || !/^[a-z-]+$/.test(name)) return;
    loaded.add(name);
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = new URL(`themes/${name}.css`, house.href).href;
    link.setAttribute("blocking", "render");
    document.head.appendChild(link);
  };

  const pick = (name) => {
    if (!values.includes(name)) return false;
    picker.value = name;
    load(name);
    return true;
  };

  const save = (name) => {
    try {
      localStorage.setItem("style", name);
    } catch {
      // Storage blocked: the picker still works for this page.
    }
  };

  // Readers who never picked a style get Star Trek on Star Trek Day,
  // Diablo for Halloween week, Doctor Who on its birthday (ADR
  // 2026-10-21-doctor-who-style, ADR 2026-10-22-star-trek-style), Holiday
  // for December, and on April Fools' Day any style in the menu at random,
  // drawn again on each page (ADR
  // 2026-10-22-ft-solarized-and-editor-styles), without any of it being
  // saved.
  const anyStyle = () => {
    const styles = values.filter((value) => !/^(auto|light|dark)$/.test(value));
    return styles[Math.floor(Math.random() * styles.length)];
  };
  const season = () => {
    const now = new Date();
    const month = now.getMonth() + 1;
    const day = now.getDate();
    if (month === 4 && day === 1) return anyStyle();
    if (month === 9 && day === 8) return "star-trek";
    if (month === 10 && day >= 24) return "diablo";
    if (month === 11 && day === 23) return "doctor-who";
    if (month === 12 && day <= 26) return "holiday";
    return null;
  };

  // A shared link can name a style, as ?style=diablo. Following it counts
  // as picking that style, so it's saved and stays on as the reader moves
  // between pages; a name that isn't in the menu is ignored
  // (ADR 2026-10-22-style-share-links).
  const linked = new URLSearchParams(location.search).get("style");
  if (linked && pick(linked)) {
    save(linked);
  } else {
    let saved = null;
    try {
      saved = localStorage.getItem("style");
    } catch {
      // As above.
    }
    pick(saved || season());
  }

  picker.addEventListener("change", () => {
    load(picker.value);
    save(picker.value);
  });
})();
