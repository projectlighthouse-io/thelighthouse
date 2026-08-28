/**
 * The rust api, as seen from nitro.
 *
 * Over loopback, never through caddy — the api binds 127.0.0.1 and caddy is the
 * public listener. An SSR fetch therefore never leaves the container, which is
 * also why Cloudflare plays no part in caching it: the edge only ever sees the
 * document nitro returns.
 *
 * These calls happen server side only. The browser talks to `/_api/*`, which is
 * nitro's own surface; it has no route to the rust api at all.
 */
const base = (): string =>
  useRuntimeConfig().apiBase || 'http://127.0.0.1:9000'

/**
 * A GET against the api, with its 404 turned into nitro's.
 *
 * Anything else — the api being down, a 500 — is left to throw as it is. A
 * page that renders "not found" because the backend was restarting is worse
 * than one that errors honestly.
 */
export async function fromApi<T>(path: string): Promise<T> {
  try {
    // Cast because `$fetch<T>` returns `TypedInternalResponse<..., T>`, which
    // resolves to T for a concrete type but not for a type parameter — the
    // compiler cannot prove the two agree while T is still open. The runtime
    // value is exactly what T describes; only the generic is unprovable.
    return await $fetch<T>(`${base()}${path}`) as T
  }
  catch (error: unknown) {
    const status = (error as { status?: number, statusCode?: number })?.status
      ?? (error as { statusCode?: number })?.statusCode

    if (status === 404) {
      throw createError({ statusCode: 404, statusMessage: 'Not found' })
    }

    throw error
  }
}

/** What the rust api returns. Snake case, because that is the wire format. */
export interface ApiPrice {
  amount: number
  currency: string
}

export interface ApiBookSummary {
  slug: string
  title: string
  description: string | null
  thumbnail_url: string | null
  price: ApiPrice
  lesson_count: number
  first_lesson: string | null
}

export interface ApiLessonSummary {
  slug: string
  title: string
  description: string | null
  sort_order: number
  /** Whether anything in this lesson sits behind the paywall. Says only that
   *  something is withheld, never what. */
  has_paid_part: boolean
}

export interface ApiChapter {
  id: number
  title: string
  lessons: ApiLessonSummary[]
}

export interface ApiBookDetail extends ApiBookSummary {
  chapters: ApiChapter[]
  seo: Record<string, string | null>
}

export interface ApiLesson {
  slug: string
  title: string
  description: string | null
  chapter_id: number
  sort_order: number
  html: string
  toc: { id: string, text: string }[]
  read_minutes: number
  has_paid_part: boolean
  remaining_sections: number
  position: number
  total: number
  percent: number
  book: { slug: string, title: string, thumbnail_url: string | null }
  previous: { slug: string, title: string } | null
  next: { slug: string, title: string } | null
  seo: Record<string, string | null>
}

/**
 * Minor units to what a price tag says. Zero is "Free", not "$0.00".
 *
 * Display only — every calculation stays in minor units on the rust side, so
 * nothing here can introduce a rounding error into what someone is charged.
 */
export function priceLabel(price: ApiPrice): string | null {
  if (price.amount === 0) {
    return null
  }

  return `$${(price.amount / 100).toFixed(2).replace(/\.00$/, '')}`
}
