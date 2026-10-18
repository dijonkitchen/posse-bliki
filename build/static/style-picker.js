// Remembers the style picked in the header from page to page, the only
// script on the site (ADR 2026-10-18-style-picker). Without it the picker
// still restyles the page you're on, through CSS :has(); it just forgets.
// It runs where it sits, right after the picker, so a remembered style is
// in place before the page below it is drawn.
(function () {
  var picker = document.getElementById("style");
  if (!picker) return;
  try {
    var saved = localStorage.getItem("style");
    for (var i = 0; i < picker.options.length; i++) {
      if (picker.options[i].value === saved) picker.value = saved;
    }
  } catch (e) {
    // Storage blocked: the picker still works for this page.
  }
  picker.addEventListener("change", function () {
    try {
      localStorage.setItem("style", picker.value);
    } catch (e) {
      // As above.
    }
  });
})();
