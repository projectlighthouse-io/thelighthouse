/**
 * Gate for pages that need a signed-in reader.
 *
 * Note this is a *convenience*, not a security boundary. Route middleware runs
 * in the browser on client-side navigation and can be bypassed. Anything that
 * actually matters — paid lesson bodies, admin data — is gated by rust, which
 * is why the api never trusts the frontend for entitlement.
 *
 * Every page using it is `ssr: false`, so this only ever runs client side and
 * the session is a real answer by the time it decides. On the server it does
 * nothing, deliberately: `useAuth` cannot resolve there, and redirecting on an
 * unresolved session would sign everyone out at the door.
 */
export default defineNuxtRouteMiddleware(async (to) => {
  if (import.meta.server) return

  const { isSignedIn, load } = useAuth()

  await load()

  if (isSignedIn.value) return

  return navigateTo({
    path: '/login',
    // so the reader lands back where they were trying to go
    query: { redirect: to.fullPath },
  })
})
