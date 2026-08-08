/**
 * Thin wrapper around the static asset store.
 *
 * Cloudflare already serves everything in ./static/ on its own, so this Worker
 * only exists to do three things the plain asset handler won't:
 *
 *   1. Guarantee `application/wasm`. Without it the browser falls back from
 *      streaming instantiation to buffering the whole module first.
 *   2. Redirect a bare mount path to its trailing-slash form, so the relative
 *      URLs in index.html ("./pkg/flappy.js") resolve correctly.
 *   3. Cache the wasm/JS/sprites hard, and the HTML not at all.
 */

const IMMUTABLE = "public, max-age=31536000, immutable";

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;

    // "/flappy" -> "/flappy/" so relative asset URLs don't resolve one level up.
    if (/^\/[^/.]+$/.test(path) && path !== "/") {
      const probe = await env.ASSETS.fetch(new Request(`${url.origin}${path}/index.html`));
      if (probe.ok) {
        return Response.redirect(`${url.origin}${path}/${url.search}`, 301);
      }
    }

    const response = await env.ASSETS.fetch(request);
    if (!response.ok) return response;

    const headers = new Headers(response.headers);

    if (path.endsWith(".wasm")) {
      headers.set("Content-Type", "application/wasm");
    }

    // The build output is not content-hashed, so cache on ETag revalidation
    // rather than blindly forever. Sprites never change, so those can be pinned.
    if (path.startsWith("/assets/")) {
      headers.set("Cache-Control", IMMUTABLE);
    } else if (path.endsWith(".wasm") || path.endsWith(".js")) {
      headers.set("Cache-Control", "public, max-age=0, must-revalidate");
    } else if (path.endsWith(".html") || path === "/" || path.endsWith("/")) {
      headers.set("Cache-Control", "no-cache");
    }

    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers,
    });
  },
};
