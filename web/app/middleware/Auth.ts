/**
 * Gate for pages that need a signed-in reader.
 *
 * Today it reads the preview cookie; at phase 3 the body of this file becomes
 * a GET /api/me check and nothing else changes — every page already opts in
 * via definePageMeta, so the call sites stay put.
 *
 * Note this is a *convenience*, not a security boundary. Route middleware runs
 * in the browser on client-side navigation and can be bypassed. Anything that
 * actually matters — paid lesson bodies, admin data — is gated by rust, which
 * is why the api never trusts the frontend for entitlement.
 */
export default defineNuxtRouteMiddleware((to) => {
  const { isSignedIn } = usePreviewAuth()

  if (isSignedIn.value) return

  return navigateTo({
    path: '/login',
    // so the reader lands back where they were trying to go
    query: { redirect: to.fullPath },
  })
})
