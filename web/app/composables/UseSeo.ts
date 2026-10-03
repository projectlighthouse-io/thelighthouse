export const SITE = {
  url: 'https://projectlighthouse.io',
  name: 'projectlighthouse',
  ogImage: '/projectlighthouse.png',
  twitter: '@thearyanahmed',
} as const

export interface SeoInput {
  title: string
  description: string
  /** og:type — 'website' for listings, 'article' for anything with a body */
  type?: 'website' | 'article'
  /** absolute path from an image in /public, or a full URL for remote art */
  image?: string
  publishedAt?: string
  /** private pages: keep them out of the index entirely */
  noindex?: boolean
  /** the content's language. Anything but English is its own url, `?lang=` */
  lang?: string
}

const absolute = (path: string): string =>
  path.startsWith('http') ? path : `${SITE.url}${path.startsWith('/') ? path : `/${path}`}`

/**
 * One place that sets the meta every page needs.
 *
 * Canonical matters more here than usual: the legacy /{locale}/ URLs 301 to
 * these paths, so search engines are actively re-pointing at them. A missing or
 * wrong canonical during that window sends the redirect's equity nowhere.
 */
export function useSeo(input: MaybeRefOrGetter<SeoInput>) {
  const route = useRoute()

  const resolved = computed(() => toValue(input))
  // A translation is a page of its own, so it is its own canonical — pointing
  // it at the English url would tell search engines to drop it.
  const canonical = computed(() => {
    const lang = resolved.value.lang
    const query = lang && lang !== 'en' ? `?lang=${lang}` : ''

    return `${SITE.url}${route.path === '/' ? '' : route.path}${query}`
  })
  const image = computed(() => absolute(resolved.value.image ?? SITE.ogImage))

  useSeoMeta({
    title: () => resolved.value.title,
    description: () => resolved.value.description,

    ogType: () => resolved.value.type ?? 'website',
    ogTitle: () => resolved.value.title,
    ogDescription: () => resolved.value.description,
    ogUrl: () => canonical.value,
    ogImage: () => image.value,
    ogSiteName: SITE.name,

    twitterCard: 'summary_large_image',
    twitterTitle: () => resolved.value.title,
    twitterDescription: () => resolved.value.description,
    twitterImage: () => image.value,
    twitterSite: SITE.twitter,

    articlePublishedTime: () => resolved.value.publishedAt,

    // noindex still allows crawling, so internal links are followed and the
    // rest of the site still benefits from them
    robots: () => (resolved.value.noindex ? 'noindex, follow' : 'index, follow'),
  })

  useHead({
    htmlAttrs: { lang: () => resolved.value.lang ?? 'en' },
    link: [{ rel: 'canonical', href: () => canonical.value }],
  })

  return { canonical }
}

/** Adds a JSON-LD block. Nuxt dedupes by the `key`, so re-renders stay clean. */
export function useJsonLd(key: string, data: MaybeRefOrGetter<Record<string, unknown>>) {
  useHead({
    script: [
      {
        key,
        type: 'application/ld+json',
        innerHTML: computed(() => JSON.stringify({ '@context': 'https://schema.org', ...toValue(data) })),
      },
    ],
  })
}
