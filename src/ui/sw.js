// Clipper Service Worker — QR cache with TTL & offline caching
const CACHE_NAME = "clipper-cache-v1";
const QR_CACHE_NAME = "clipper-qr-v1";
const QR_TTL_MS = 5 * 60 * 1000; // 5-minute TTL

self.addEventListener("install", (event) => {
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys
          .filter((k) => k !== CACHE_NAME && k !== QR_CACHE_NAME)
          .map((k) => caches.delete(k))
      );
    }).then(() => self.clients.claim())
  );
});

self.addEventListener("message", (event) => {
  if (event.data && event.data.type === "INVALIDATE_QR") {
    caches.open(QR_CACHE_NAME).then((cache) => {
      cache.delete("/pair.png");
    });
  }
});

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);

  // Handle QR code caching with TTL
  if (url.pathname === "/pair.png" && event.request.method === "GET") {
    event.respondWith(
      caches.open(QR_CACHE_NAME).then(async (cache) => {
        const cachedRes = await cache.match(event.request);
        const cachedTime = cachedRes ? cachedRes.headers.get("x-clipper-cached-at") : null;

        if (cachedRes && cachedTime) {
          const age = Date.now() - parseInt(cachedTime, 10);
          if (age < QR_TTL_MS) {
            return cachedRes;
          }
        }

        // Cache expired or missing -> fetch fresh
        try {
          const freshRes = await fetch(event.request);
          if (freshRes && freshRes.status === 200) {
            const copy = freshRes.clone();
            const headers = new Headers(copy.headers);
            headers.set("x-clipper-cached-at", Date.now().toString());

            const blob = await copy.blob();
            const modifiedRes = new Response(blob, {
              status: copy.status,
              statusText: copy.statusText,
              headers: headers,
            });

            await cache.put(event.request, modifiedRes);
          }
          return freshRes;
        } catch (err) {
          if (cachedRes) return cachedRes;
          throw err;
        }
      })
    );
    return;
  }

  // Pass-through for all other requests
  event.respondWith(
    fetch(event.request).catch(async () => {
      const match = await caches.match(event.request);
      return match || new Response("Offline", { status: 503, statusText: "Service Unavailable" });
    })
  );
});
