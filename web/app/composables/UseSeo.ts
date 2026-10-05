export const SITE = {
  url: 'https://projectlighthouse.io',
  /** As search results, social cards and structured data spell it. The
   *  wordmark on the page stays lowercase `projectlighthouse`. */
  name: 'Project Lighthouse',
  /** The site's share card, from the brand assets: 1200×630. */
  ogImage: '/og-image.png',
  ogImageWidth: 1200,
  ogImageHeight: 630,
  ogImageAlt: 'Project Lighthouse: fundamentals of software engineering',
  /** The square icon, for structured data that asks for a logo. */
  logo: '/icon-512.png',
  twitter: '@thearyanahmed',
} as const

export interface SeoInput {
  title: string
  description: string
  /** The title exactly as given, without the site's name after it. For the
   *  home page, whose title already leads with the name. */
  bare?: boolean
  /** og:type — 'website' for listings, 'book' for a book, 'article' for a
   *  lesson or a post */
  type?: 'website' | 'article' | 'book'
  /**
   * absolute path from an image in /public, or a full URL for remote art.
   * Leave it out unless the image is known to suit a share card (1200px wide
   * or more): the default is the site's card, whose size is known.
   */
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
  // Only the site's own card has a size we know; a page's own art does not
  // claim one rather than claim a wrong one.
  const isDefaultImage = computed(() => resolved.value.image === undefined)
  // Every page gets the site's name and a description that fits a results
  // page, whatever it passed in — see `utils/Seo`.
  const title = computed(() =>
    resolved.value.bare ? resolved.value.title : pageTitle(resolved.value.title))
  const description = computed(() => metaDescription(resolved.value.description))

  useSeoMeta({
    title: () => title.value,
    description: () => description.value,

    ogType: () => resolved.value.type ?? 'website',
    ogTitle: () => title.value,
    ogDescription: () => description.value,
    ogUrl: () => canonical.value,
    ogImage: () => image.value,
    ogImageWidth: () => (isDefaultImage.value ? SITE.ogImageWidth : undefined),
    ogImageHeight: () => (isDefaultImage.value ? SITE.ogImageHeight : undefined),
    ogImageAlt: () => (isDefaultImage.value ? SITE.ogImageAlt : title.value),
    ogSiteName: SITE.name,

    twitterCard: 'summary_large_image',
    twitterTitle: () => title.value,
    twitterDescription: () => description.value,
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
