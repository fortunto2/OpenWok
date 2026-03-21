// Minimal Service Worker — caches app shell (WASM + JS + CSS) for offline cold start.
// NOT a PWA — no manifest, no install prompt. Just cache.

const CACHE_NAME = 'openwok-v1';

// Cache app shell on install
self.addEventListener('install', (event) => {
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      // Cache the root page — dx generates the HTML with script tags
      return cache.add('/');
    })
  );
});

// Clean old caches on activate
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)))
    )
  );
  self.clients.claim();
});

// Network-first for API, cache-first for assets
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // API calls: network only (offline handled by app's outbox)
  if (url.pathname.startsWith('/api/')) {
    return;
  }

  // Assets (WASM, JS, CSS): cache-first, fallback to network
  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) return cached;
      return fetch(event.request).then((response) => {
        // Cache successful responses
        if (response.ok) {
          const clone = response.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
        }
        return response;
      });
    }).catch(() => {
      // If both cache and network fail, return cached root for navigation
      if (event.request.mode === 'navigate') {
        return caches.match('/');
      }
    })
  );
});
