/**
 * Who the reader is, according to rust.
 *
 * The session lives in an HttpOnly cookie this app cannot read, so the only way
 * to know is to ask: GET /api/auth/session answers with the reader or with
 * null. Nothing here is a security boundary — it decides what chrome to draw.
 * Entitlement is decided in rust, against the same cookie.
 *
 * **The fetch is client-side on purpose.** Anonymous pages are prerendered and
 * edge-cached; if this ran during SSR the HTML would differ per reader and
 * either leak one reader's name into a shared cache or make every page
 * uncacheable. So the chrome is an island: the page renders signed-out, then
 * fills in. Signed-in pages are `ssr: false` anyway, so they lose nothing.
 */

export interface AuthUser {
  /** `provider:id` — stable across name and email changes. */
  sub: string
  provider: 'github' | 'google'
  name: string | null
  email: string | null
  avatar: string | null
}

/**
 * The in-flight request, shared so ten components mounting at once make one
 * call. Module scope is safe *because* this only ever runs in the browser,
 * where the module belongs to one reader — on the server it would be shared
 * across every request, which is why `load` refuses to run there.
 */
let inFlight: Promise<void> | null = null

/**
 * Whether the last sign-in is still believed to be live.
 *
 * Read from `lh_reader`, a cookie rust sets beside the session and clears with
 * it. Not authentication — the value is always `1` and grants nothing. It exists
 * so the header knows which shape to draw on its *first* render, rather than
 * showing a join button to a signed-in reader for the length of a round trip.
 *
 * A cookie rather than localStorage, which is what this was first: localStorage
 * is only written once a session fetch has already succeeded, so the first load
 * after signing in — and after any deploy that cleared it — flashed anyway. The
 * cookie arrives with the redirect that signs you in, so it is right from the
 * very first paint.
 *
 * Stale is self-correcting and fails the safe way: an expired session shows your
 * avatar until `/api/auth/session` replies, then falls back to join. The reverse
 * is what looked broken.
 */
function readerHint(): boolean {
  if (import.meta.server) return false

  return document.cookie.split(';').some(pair => pair.trim() === 'lh_reader=1')
}

/** First letters of the first and last word, for the avatar fallback. */
function initialsOf(user: AuthUser | null): string {
  const name = user?.name?.trim()
  if (!name) return user?.email?.[0]?.toUpperCase() ?? '?'

  const words = name.split(/\s+/)
  const first = words[0]?.[0] ?? ''
  const last = words.length > 1 ? (words[words.length - 1]?.[0] ?? '') : ''

  return (first + last).toUpperCase()
}

export function useAuth() {
  const user = useState<AuthUser | null>('auth.user', () => null)
  // Distinct from `user === null`, which cannot tell "signed out" from "not
  // asked yet". The chrome uses it to avoid flashing a join button at somebody
  // who is in fact signed in.
  const resolved = useState<boolean>('auth.resolved', () => false)

  const hint = useState<boolean>('auth.hint', () => false)

  // Read here, not in the `useState` initialiser. That initialiser runs on the
  // *server*, where there are no cookies to read from `document`, and its result
  // is serialised into the payload — so the client hydrates `false` and never
  // asks. The symptom is the flash this exists to remove, arriving anyway.
  //
  // This runs during setup, before the first render.
  if (import.meta.client && !resolved.value) {
    hint.value = readerHint()
  }

  /**
   * What the chrome should draw *now*.
   *
   * Before the session answers, this is the remembered answer; after, it is the
   * real one. The header has to render something on first paint, and rendering
   * "signed out" at a signed-in reader is the flash this exists to remove.
   */
  const looksSignedIn = computed<boolean>(() =>
    resolved.value ? user.value !== null : hint.value,
  )

  async function load(force = false): Promise<void> {
    // SSR renders every reader as anonymous. See the note at the top.
    if (import.meta.server) return
    if (resolved.value && !force) return
    if (inFlight && !force) return inFlight

    inFlight = $fetch<AuthUser | null>('/api/auth/session')
      .then((reader) => {
        user.value = reader ?? null
        // `user.value`, not `reader`: $fetch answers undefined for an empty
        // body, and `undefined !== null` would read as signed in.
        hint.value = user.value !== null
      })
      .catch(() => {
        // The api being unreachable is not a signed-in reader. Failing to
        // anonymous keeps the chrome honest instead of leaving it half-drawn.
        //
        // The hint is left alone: a request that never arrived is not evidence
        // of being signed out, and forgetting on a flaky connection would put
        // the flash back on the next load. Rust owns the cookie either way.
        user.value = null
      })
      .finally(() => {
        resolved.value = true
        inFlight = null
      })

    return inFlight
  }

  async function signOut(): Promise<void> {
    // POST, so a prefetch or an <img> on another site cannot sign anyone out.
    // The cookie is cleared by rust; clearing local state is just the redraw.
    await $fetch('/api/auth/logout', { method: 'POST' }).catch(() => {})

    user.value = null
    resolved.value = true
    // Rust clears `lh_reader` in the same response; this is the local redraw.
    hint.value = false

    await navigateTo('/')
  }

  /** Sends the browser to rust, which owns the whole OAuth dance. */
  function signInUrl(provider: 'github' | 'google', redirect?: string): string {
    const query = redirect ? `?redirect=${encodeURIComponent(redirect)}` : ''

    return `/api/auth/${provider}${query}`
  }

  return {
    user,
    resolved,
    looksSignedIn,
    isSignedIn: computed<boolean>(() => user.value !== null),
    initials: computed<string>(() => initialsOf(user.value)),
    load,
    signOut,
    signInUrl,
  }
}
