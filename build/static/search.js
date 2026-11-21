// Searches every note and page in your browser, for the search page only
// (ADR 2026-11-21-search-page). The build writes what there is to search,
// as plain text, to search.json beside this file. It's fetched the first
// time you search, from this site and nothing else, so what you type never
// leaves your device; the offline worker keeps it like any page you read.
// Without this script the form stays disabled and the page points to Notes.
// It runs after the page is parsed (defer), so the form is there.
// Modern syntax, like the style picker: helpers are arrows, so each one is
// defined above its first use (ADR 2026-10-28-modern-js-style-picker).
(() => {
  "use strict";
  const index = new URL("search.json", document.currentScript.src);
  const form = document.getElementById("search");
  const box = form.elements.q;
  const status = document.getElementById("search-status");
  const results = document.createElement("ol");
  results.className = "search-results";
  status.after(results);

  // Case and accents don't matter, so "cafe" finds "Café". Each accented
  // letter folds to one letter, so a place in the folded text is the same
  // place in the original.
  const fold = (text) => text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
  const words = (text) => fold(text).split(/[^\p{L}\p{N}]+/u).filter(Boolean);

  // Where `word` first starts a word of folded `text`, or -1. Matching word
  // starts lets "obsid" find "Obsidian" while "rust" skips "trust".
  const find = (text, word) => {
    for (let at = text.indexOf(word); at >= 0; at = text.indexOf(word, at + 1)) {
      if (at === 0 || !/[\p{L}\p{N}]/u.test(text[at - 1])) return at;
    }
    return -1;
  };

  let entries = null;
  const load = async () => {
    if (!entries) {
      const response = await fetch(index);
      if (!response.ok) throw new Error(`search.json: ${response.status}`);
      entries = (await response.json()).map((entry) => ({
        ...entry,
        foldedTitle: fold(entry.title),
        foldedText: fold(entry.text),
      }));
    }
    return entries;
  };

  // A line or two of the text around the first word found in it, with that
  // word marked, or the start of the text when only the title matched.
  // Built from text nodes, never parsed as HTML.
  const snippet = (entry, query) => {
    const text = entry.text;
    const word = query.find((w) => find(entry.foldedText, w) >= 0) ?? "";
    const at = word ? find(entry.foldedText, word) : 0;
    const end = at + word.length;
    const from = at > 60 ? Math.min(at, text.indexOf(" ", at - 60) + 1) : 0;
    const cut = text.indexOf(" ", end + 100);
    const to = cut < 0 ? text.length : cut;
    const mark = document.createElement("mark");
    mark.textContent = text.slice(at, end);
    const p = document.createElement("p");
    p.append(from > 0 ? "… " : "", text.slice(from, at), word ? mark : "", text.slice(end, to), to < text.length ? " …" : "");
    return p;
  };

  const show = (found, q, query) => {
    const items = found.map((entry) => {
      const link = document.createElement("a");
      // The index has canonical addresses; the path also works on a local
      // build and under a sub-path, where the build sets the base URL.
      link.href = new URL(entry.url).pathname;
      link.textContent = entry.title;
      const li = document.createElement("li");
      li.append(link, snippet(entry, query));
      return li;
    });
    results.replaceChildren(...items);
    const count = found.length === 1 ? "1 result" : `${found.length || "No"} results`;
    status.textContent = `${count} for “${q}”.`;
  };

  // Every word must match. Title matches come first; otherwise the index's
  // order, which is the order of Notes, then the other pages.
  const search = async () => {
    const q = box.value.trim();
    const url = new URL(location.href);
    if (q) url.searchParams.set("q", q);
    else url.searchParams.delete("q");
    history.replaceState(null, "", url);
    const query = words(q);
    if (!query.length) {
      results.replaceChildren();
      status.textContent = "";
      return;
    }
    let all;
    try {
      all = await load();
    } catch {
      status.textContent = "Search couldn't load. Check your connection and try again.";
      return;
    }
    // Typing went on while the index loaded: that later search shows instead.
    if (box.value.trim() !== q) return;
    const inTitle = (entry) => query.every((w) => find(entry.foldedTitle, w) >= 0);
    const found = all.filter((entry) =>
      query.every((w) => find(entry.foldedTitle, w) >= 0 || find(entry.foldedText, w) >= 0),
    );
    show([...found.filter(inTitle), ...found.filter((entry) => !inTitle(entry))], q, query);
  };

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    search();
  });
  box.addEventListener("input", search);
  box.value = new URLSearchParams(location.search).get("q") ?? "";
  for (const control of form.elements) control.disabled = false;
  if (box.value) search();
})();
