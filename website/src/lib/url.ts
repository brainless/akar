// Prefix a root-relative path with the configured base (import.meta.env.BASE_URL)
// so internal links and assets work when the site is served from a subpath —
// e.g. a GitHub Pages project site at https://<user>.github.io/<repo>/.
//
// Astro auto-prefixes imported assets (astro:assets) and Astro.url, but NOT
// hardcoded href/src strings, so those go through withBase(). It is idempotent
// and leaves external URLs, anchors and already-prefixed paths untouched.
const BASE = import.meta.env.BASE_URL.replace(/\/$/, '');

export function withBase(path: string): string {
  if (!path) return path;
  // Absolute URL, protocol-relative, anchor, or mailto/tel — leave as-is.
  if (
    /^([a-z][a-z0-9+.-]*:)?\/\//i.test(path) ||
    path.startsWith('#') ||
    /^(mailto:|tel:)/i.test(path)
  ) {
    return path;
  }
  if (!path.startsWith('/')) return path; // relative path — leave
  if (BASE && (path === BASE || path.startsWith(BASE + '/'))) return path; // already prefixed
  return BASE + path;
}
