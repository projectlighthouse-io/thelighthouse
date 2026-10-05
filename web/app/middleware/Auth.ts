/**
 * Gate for pages that need a signed-in reader.
 *
 * Note this is a *convenience*, not a security boundary. Route middleware runs
 * in the browser on client-side navigation and can be bypassed. Anything that
 * actually matters — paid lesson bodies, admin data — is gated by rust, which
 * is why the api never trusts the frontend for entitlement.
 */
export default defineNuxtRouteMiddleware(async (to) => {
  // Every page that opts in is `ssr: false`, so this only ever runs in the
  // browser. Bailing on the server keeps it correct if one ever stops being:
  // the session cannot be resolved there, and bouncing to /login is the wrong
  // answer to "we have not asked yet".
  if (import.meta.server) return

  const { isSignedIn, resolve } = useReader()

  await resolve()

  if (isSignedIn.value) return

  return navigateTo({
    path: '/login',
    // so the reader lands back where they were trying to go
    query: { redirect: to.fullPath },
  })
})
