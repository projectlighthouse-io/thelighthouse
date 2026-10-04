/** What `/api/billing/access` answers: the track, and every book it opens. */
export interface Access {
  track: string | null
  books: { slug: string }[]
}

/**
 * Which books the signed-in reader may read, asked once per page load.
 *
 * Browser only, like everything that carries the session cookie — the pages
 * around it are cached for everyone. Anonymous readers own nothing and never
 * ask. Display only: what a lesson actually shows is decided by rust.
 */
export function useAccess() {
  const access = useState<Access | null>('billing.access', () => null)
  const resolved = useState<boolean>('billing.access.resolved', () => false)

  const { isSignedIn, resolve } = useReader()

  async function load(): Promise<void> {
    if (import.meta.server || resolved.value) return

    await resolve()

    if (isSignedIn.value) {
      access.value = await $fetch<Access | null>('/api/billing/access').catch(() => null)
    }

    resolved.value = true
  }

  const owns = (slug: string): boolean =>
    access.value?.books.some(book => book.slug === slug) ?? false

  return { access, resolved, load, owns }
}
