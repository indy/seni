/*
 *  Seni
 *  Copyright (C) 2019 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

const PRECACHE = 'precache-v5';
const RUNTIME = 'runtime';

// NOTE: when releasing set DONT_CACHE_SOME_URLS to false and increment the PRECACHE
const DONT_CACHE_SOME_URLS = true;
const NO_CACHE_URLS = [
  '/client.js',
  '/client_bg.wasm',
  '/index.js',
  '/worker.js',
  '/gallery',
  '/shader/main-vert.glsl',
  '/shader/main-frag.glsl',
  '/shader/blit-vert.glsl',
  '/shader/blit-frag.glsl',
];

// A list of local resources we always want to be cached.
const PRECACHE_URLS = [
  '/css/bones.css',
  '/lib/codemirror/codemirror.js',
  '/lib/codemirror/closebrackets.js',
  '/lib/codemirror/matchbrackets.js',
  '/favicon.ico',
  '/img/spinner.gif'
];

// The install handler takes care of precaching the resources we always need.
self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(PRECACHE)
      .then(cache => cache.addAll(PRECACHE_URLS))
      .then(self.skipWaiting())
  );
});

// The activate handler takes care of cleaning up old caches.
self.addEventListener('activate', event => {
  const currentCaches = [PRECACHE, RUNTIME];
  event.waitUntil(
    caches.keys().then(cacheNames => {
      return cacheNames.filter(cacheName => !currentCaches.includes(cacheName));
    }).then(cachesToDelete => {
      return Promise.all(cachesToDelete.map(cacheToDelete => {
        return caches.delete(cacheToDelete);
      }));
    }).then(() => self.clients.claim())
  );
});

// The fetch handler serves responses for same-origin resources from a cache.
// If no response is found, it populates the runtime cache with the response
// from the network before returning it to the page.
self.addEventListener('fetch', event => {
  // Skip cross-origin requests, like those for Google Analytics.
  if (event.request.url.startsWith(self.location.origin)) {
    // during development don't cache files that are constantly changing
    // currently this includes any files in the NO_CACHE_URLS array and
    // every piece script (a url beginning with gallery/{id})
    //
    if(DONT_CACHE_SOME_URLS) {
      let gallery_item_re = /\/gallery\/\d+/;
      const relative = event.request.url.substr(self.location.origin.length);
      if(NO_CACHE_URLS.includes(relative) || relative.match(gallery_item_re)) {
        // console.log(`fetching but not caching ${relative}`);
        return fetch(event.request).then(response => {
          return response;
        });
      }
    }

    // cache everything
    event.respondWith(
      caches.match(event.request).then(cachedResponse => {
        if (cachedResponse) {
          // console.log(`in cache: ${event.request.url}`);
          return cachedResponse;
        }

        return caches.open(RUNTIME).then(cache => {
          return fetch(event.request).then(response => {
            // Put a copy of the response in the runtime cache.
            return cache.put(event.request, response.clone()).then(() => {
              // console.log(`added to cache: ${event.request.url}`);
              return response;
            });
          });
        });
      })
    );
  }
});
