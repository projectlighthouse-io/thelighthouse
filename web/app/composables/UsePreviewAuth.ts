export interface PreviewUser {
  name: string
  email: string
  initials: string
}

/**
 * A stand-in for real auth so the signed-in chrome can be looked at before
 * OAuth exists. Backed by a cookie purely so SSR renders the same thing the
 * client does while previewing.
 *
 * This is not how auth will work. The real design keeps signed-in chrome as a
 * client-side island calling GET /api/me, precisely so anonymous HTML stays
 * byte-identical and cacheable — see docs/rebuild.md. Delete this composable
 * when phase 3 lands.
 */
export function usePreviewAuth() {
  const cookie = useCookie<string | null>('preview_auth', {
    default: () => null,
    sameSite: 'lax',
    // dev-only, so no need for secure/httpOnly ceremony
  })

  const user = computed<PreviewUser | null>(() =>
    cookie.value
      ? { name: 'Aryan Ahmed', email: 'thearyanahmed@gmail.com', initials: 'AA' }
      : null,
  )

  const isSignedIn = computed<boolean>(() => user.value !== null)

  const signIn = (): void => {
    cookie.value = '1'
  }

  const signOut = (): void => {
    cookie.value = null
  }

  return { user, isSignedIn, signIn, signOut }
}
