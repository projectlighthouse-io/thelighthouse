/** What GET /api/auth/session returns. Snake case, because it is the wire. */
export interface Reader {
  /**
   * The `users.id`, as a string. Opaque — never parsed, only compared.
   */
  sub: string
  provider: string
  name: string
  email: string
  avatar: string | null
  /**
   * Echoed in a header on every write — see the api's `middleware::csrf`.
   * Handed out here rather than in a cookie, so it only ever reaches the
   * reader it belongs to.
   */
  csrf: string
}

/**
 * A companion to the session cookie that javascript is allowed to read. It
 * carries no identity and grants nothing; the value is always `1`.
 */
const HINT_COOKIE = 'lh_reader'

/**
 * The header every write carries — see the api's `middleware::csrf`.
 *
 * Absent when the session has not answered yet, and the api refuses the write
 * with a 403. That is the honest outcome rather than a bug to route around: a
 * write sent before we know who is writing is one that should not have been
 * sent. Callers wait for `resolve()`.
 */
export function csrfHeader(): Record<string, string> {
  const { reader } = useReader()

  return reader.value?.csrf ? { 'X-CSRF-Token': reader.value.csrf } : {}
}

/**
 * Memoised so one page load asks once however many components ask it.
 *
 * Module scope is safe only because `resolve` refuses to run on the server —
 * a promise shared across SSR requests would hand one reader's session to the
 * next visitor.
 */
let inFlight: Promise<Reader | null> | null = null

/**
 * The signed-in reader, resolved in the browser and never during SSR.
 *
 * Public pages are prerendered or edge-cached, so their HTML has to be
 * identical for everyone. Rendering the header from a session would make every
 * page per-reader and uncacheable, so the server always renders the anonymous
 * chrome and the browser corrects it — see docs/rebuild.md.
 *
 * `lh_reader` is what keeps that correction from being a visible flash: it
 * says which shape the header should be before `/api/auth/session` has
 * answered. Nothing is trusted because of it. It is a hint about chrome, and
 * every byte that matters is gated by rust against the session row.
 */
export function useReader() {
  const reader = useState<Reader | null>('reader', () => null)
  /** Whether the endpoint has answered. Until it has, the hint decides. */
  const settled = useState<boolean>('reader:settled', () => false)

  // Evaluated once, in the browser. Only consulted before `settled`, so it
  // never needs to react to the cookie changing underneath it.
  const hinted = computed<boolean>(() =>
    import.meta.client
    && document.cookie.split(';').some(pair => pair.trim() === `${HINT_COOKIE}=1`),
  )

  const isSignedIn = computed<boolean>(() =>
    settled.value ? reader.value !== null : hinted.value,
  )

  /** First letters of the first two words. Empty until the reader lands. */
  const initials = computed<string>(() =>
    (reader.value?.name ?? '')
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map(word => word.charAt(0).toUpperCase())
      .join(''),
  )

  /**
   * Ask who this is, once.
   *
   * The endpoint answers 200 with a null body for anonymous readers, so a
   * `null` here is an answer rather than a failure.
   */
  const resolve = async (): Promise<Reader | null> => {
    if (settled.value || import.meta.server) {
      return reader.value
    }

    inFlight ??= $fetch<Reader | null>('/api/auth/session')

    try {
      reader.value = await inFlight
      settled.value = true
    }
    catch {
      // Left unsettled on purpose. A failed request means "we do not know",
      // not "signed out" — settling here would draw a join button at a
      // signed-in reader every time the api hiccups, and send the route guard
      // bouncing them to /login. The hint keeps the chrome right until the
      // next attempt.
      reader.value = null
    }
    finally {
      inFlight = null
    }

    return reader.value
  }

  /**
   * Deletes the session row, not just this browser's copy of the cookie.
   *
   * Deliberately uncaught: if the request fails the reader stays signed in,
   * which is the truthful outcome — the row is still there.
   */
  const signOut = async (): Promise<void> => {
    // Signing out deletes the session row, so it is a write and carries the
    // token like every other write. Without it the api answers 403 and the
    // reader stays signed in — which is the truthful outcome, but a confusing
    // one, so the header is not optional here.
    const token = reader.value?.csrf
    if (!token) {
      return
    }

    await $fetch('/api/auth/logout', {
      method: 'POST',
      headers: { 'X-CSRF-Token': token },
    })

    reader.value = null
    settled.value = true

    await navigateTo('/')
  }

  return { reader, isSignedIn, initials, resolve, signOut }
}
