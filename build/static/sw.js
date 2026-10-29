// Keeps a copy of every page you read on this site, so you can read it again
// offline (ADR 2026-10-29-offline-reading). Online, every request still goes
// to the network first and the copy is only refreshed, so you never see a
// stale page while connected. Offline, you get the copy, or the offline page
// for one you haven't read yet. Only this site's own files are kept.
"use strict";

const CACHE = "pages";

// Kept on install: the pages in the header, so the notes index is there to
// pick from, and what they and the offline page need to draw in the house
// style. Everything else, the notes and the picked style included, is kept
// as you read it. Paths are relative to this file, so they work under a
// sub-path.
const SHELL = [
  "./",
  "notes/",
  "about/",
  "colophon/",
  "offline.html",
  "style.css",
  "style-picker.js",
  "fonts/atkinson-hyperlegible-next-latin.woff2",
  "icon.svg",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => cache.addAll(SHELL))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      for (const name of await caches.keys()) {
        if (name !== CACHE) await caches.delete(name);
      }
      // Starts the page request while the worker boots, so it costs online
      // readers nothing.
      await self.registration.navigationPreload?.enable();
      await self.clients.claim();
    })(),
  );
});

const fromCache = async (request) => {
  const navigate = request.mode === "navigate";
  // ?style= links are the same page in another style.
  const kept = await caches.match(request, { ignoreSearch: navigate });
  if (kept) return kept;
  if (navigate) return caches.match("offline.html");
  return Response.error();
};

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET" || new URL(request.url).origin !== location.origin) return;
  event.respondWith(
    (async () => {
      try {
        const response = (await event.preloadResponse) || (await fetch(request));
        // 200 only: redirects and partial responses can't be replayed.
        if (response.status === 200) {
          const copy = response.clone();
          event.waitUntil(caches.open(CACHE).then((cache) => cache.put(request, copy)));
        }
        return response;
      } catch {
        return fromCache(request);
      }
    })(),
  );
});
