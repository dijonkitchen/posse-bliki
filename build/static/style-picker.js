// Loads and remembers the style picked in the header, the only script on
// the site (ADR 2026-10-18-style-picker, ADR 2026-10-20-zen-garden-styles).
// Without it the picker still restyles the page you're on, through CSS
// :has() and the stylesheets in <noscript>; it just forgets.
// It runs where it sits, right after the picker, so a remembered style is
// in place before the page below it is drawn.
(function () {
  var picker = document.getElementById("style");
  if (!picker) return;
  var house = document.querySelector('link[rel="stylesheet"][href$="style.css"]');

  // A style's stylesheet, added the first time it's picked. Auto, Light and
  // Dark live in style.css. Render-blocking, so the page doesn't flash the
  // house style first. Only menu values ever reach here, and the URL is
  // resolved against the house sheet so it works under a sub-path.
  var loaded = { auto: true, light: true, dark: true };
  function load(name) {
    if (!house || loaded[name] || !/^[a-z-]+$/.test(name)) return;
    loaded[name] = true;
    var link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = new URL("themes/" + name + ".css", house.href).href;
    link.setAttribute("blocking", "render");
    document.head.appendChild(link);
  }

  function pick(name) {
    for (var i = 0; i < picker.options.length; i++) {
      if (picker.options[i].value === name) {
        picker.value = name;
        load(name);
        return true;
      }
    }
    return false;
  }

  function save(name) {
    try {
      localStorage.setItem("style", name);
    } catch (e) {
      // Storage blocked: the picker still works for this page.
    }
  }

  // Readers who never picked a style get Star Trek on Star Trek Day,
  // Diablo for Halloween week, Doctor Who on its birthday (ADR
  // 2026-10-21-doctor-who-style, ADR 2026-10-22-star-trek-style), Holiday
  // for December, and on April Fools' Day any style in the menu at random,
  // drawn again on each page (ADR
  // 2026-10-22-ft-solarized-and-editor-styles), without any of it being
  // saved.
  function anyStyle() {
    var styles = [];
    for (var i = 0; i < picker.options.length; i++) {
      var value = picker.options[i].value;
      if (!/^(auto|light|dark)$/.test(value)) styles.push(value);
    }
    return styles[Math.floor(Math.random() * styles.length)];
  }
  function season() {
    var now = new Date();
    var month = now.getMonth() + 1;
    var day = now.getDate();
    if (month === 4 && day === 1) return anyStyle();
    if (month === 9 && day === 8) return "star-trek";
    if (month === 10 && day >= 24) return "diablo";
    if (month === 11 && day === 23) return "doctor-who";
    if (month === 12 && day <= 26) return "holiday";
    return null;
  }

  // A shared link can name a style, as ?style=diablo. Following it counts
  // as picking that style, so it's saved and stays on as the reader moves
  // between pages; a name that isn't in the menu is ignored
  // (ADR 2026-10-22-style-share-links).
  var linked = new URLSearchParams(location.search).get("style");
  if (linked && pick(linked)) {
    save(linked);
  } else {
    var saved = null;
    try {
      saved = localStorage.getItem("style");
    } catch (e) {
      // As above.
    }
    pick(saved || season());
  }

  picker.addEventListener("change", function () {
    load(picker.value);
    save(picker.value);
  });
})();
