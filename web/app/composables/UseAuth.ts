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

  async function load(force = false): Promise<void> {
    // SSR renders every reader as anonymous. See the note at the top.
    if (import.meta.server) return
    if (resolved.value && !force) return
    if (inFlight && !force) return inFlight

    inFlight = $fetch<AuthUser | null>('/api/auth/session')
      .then((reader) => {
        user.value = reader ?? null
      })
      .catch(() => {
        // The api being unreachable is not a signed-in reader. Failing to
        // anonymous keeps the chrome honest instead of leaving it half-drawn.
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
    isSignedIn: computed<boolean>(() => user.value !== null),
    initials: computed<string>(() => initialsOf(user.value)),
    load,
    signOut,
    signInUrl,
  }
}
