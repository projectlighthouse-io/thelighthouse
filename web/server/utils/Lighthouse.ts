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
 * A GET against the api, with its 404 turned into nitro's and its silence into
 * a 502.
 *
 * `headers` is for the one case that needs them: a route forwarding the
 * reader's session cookie, because the api's answer depends on who is asking.
 * Anything forwarded this way is a response no shared cache may hold — the
 * calling route is what has to say so.
 *
 * A rejection carrying no status at all is a connection that was never made —
 * the api down, restarting, or not yet listening. Rethrown as it comes, nitro
 * reports that as a 500, which says *this* process broke. It did not: it is the
 * gateway, and its upstream did not answer. 502 says so, and is the difference
 * between reading the logs of the right process and the wrong one.
 *
 * A real 5xx from the api keeps its own status. That one is genuinely the
 * api's, and flattening it into 502 would lose which of the two failed.
 */
export async function fromApi<T>(
  path: string,
  query?: Record<string, string>,
  headers?: Record<string, string>,
): Promise<T> {
  try {
    // Cast because `$fetch<T>` returns `TypedInternalResponse<..., T>`, which
    // resolves to T for a concrete type but not for a type parameter — the
    // compiler cannot prove the two agree while T is still open. The runtime
    // value is exactly what T describes; only the generic is unprovable.
    return await $fetch<T>(`${base()}${path}`, { query, headers }) as T
  }
  catch (error: unknown) {
    const status = (error as { status?: number, statusCode?: number })?.status
      ?? (error as { statusCode?: number })?.statusCode

    if (status === 404) {
      throw createError({ statusCode: 404, statusMessage: 'Not found' })
    }

    if (status === undefined) {
      throw createError({
        statusCode: 502,
        statusMessage: 'The api is not answering',
      })
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
  /** The tracks the book is on, and its position in each — `{go: 4, rust: 3}`.
   *  From `book.yaml`; empty for an untracked book. */
  tracks: Record<string, number>
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
  /** The book page's slideshow, in order. Not on the summary — a listing has
   *  no use for seven image urls a book. */
  images: string[]
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
  /** `locked` is per entry: the contents list describes the whole lesson to
   *  everybody, and marks which entries this reader cannot reach. */
  toc: { id: string, text: string, locked: boolean }[]
  read_minutes: number
  has_paid_part: boolean
  /** Whether this reader may read the paid half. A fact about the reader, not
   *  about the lesson — the api needs the session cookie to answer it, and
   *  `has_paid_part` is the fact about the lesson beside it. */
  unlocked: boolean
  remaining_sections: number
  position: number
  total: number
  percent: number
  book: { slug: string, title: string, thumbnail_url: string | null }
  previous: { slug: string, title: string } | null
  next: { slug: string, title: string } | null
  seo: Record<string, string | null>
  /** The language `html` is in — what `?lang=` asked for, or `en`. */
  locale: string
  /** Every language this lesson is written in. */
  locales: string[]
}

/**
 * "There is more here, and this reader cannot see it."
 *
 * Two facts, and both routes that answer it need the same pair: `has_paid_part`
 * is about the *lesson* — something in it is withheld — and `unlocked` is about
 * the *reader*. They were combined separately in each route and drifted: the
 * contents list used the lesson half alone, so it went on calling lessons paid
 * to somebody who opens them and reads the whole thing.
 *
 * **An absent `unlocked` is not unlocked.** The book listing is the same bytes
 * for everybody and is held at the edge, so the api answers it without a
 * session and there is no reader half to have. Locked is the honest answer for
 * a response that cannot know, and the page asks again from the browser — where
 * the cookie goes — to find out otherwise.
 */
export function isLocked(
  lesson: { has_paid_part: boolean, unlocked?: boolean },
): boolean {
  return lesson.has_paid_part && !lesson.unlocked
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

export interface ApiFeature {
  title: string
  description: string | null
  icon: string | null
}

export interface ApiProjectSummary {
  id: string | null
  slug: string
  name: string
  headline: string | null
  short_description: string | null
  difficulty: string | null
  runner_image: string | null
  /** A challenge has no companion book: the tasks are the whole thing. It is
   *  what the projects page splits its two tabs on. */
  is_challenge: boolean
  is_featured: boolean
  featured_order: number
  show_tasks: boolean
  unlock_mode: 'open' | 'sequential'
  related_book_slug: string | null
  task_count: number
}

export interface ApiProjectTask {
  slug: string
  title: string
  sort_order: number
  points: number
  is_free: boolean
}

export interface ApiProjectPage extends ApiProjectSummary {
  long_description: string | null
  features: ApiFeature[]
  /** The project's long-form markdown, unrendered. */
  overview: string | null
  tasks: ApiProjectTask[]
}

export interface ApiTaskRef {
  slug: string
  title: string
}

export interface ApiTaskPage {
  slug: string
  title: string
  sort_order: number
  points: number
  is_free: boolean
  /** The brief, already rendered. A terminal gets the markdown; a browser gets
   *  this. */
  html: string
  position: number
  total: number
  project: { slug: string, name: string }
  previous: ApiTaskRef | null
  next: ApiTaskRef | null
}

/**
 * An article, as `GET /api/articles` returns it. Snake case, because that is
 * the wire.
 *
 * `body` is markdown as its author typed it — rust renders nothing, so it
 * arrives raw and `SafeMarkdown` is what turns it into html.
 */
export interface ApiArticle {
  id: number
  slug: string
  title: string
  /** The line under the title. Author-written and required. */
  subtitle: string
  /** Every topic the article is filed under; at least one. */
  topics: string[]
  body: string
  author: string
  author_username: string | null
  created_at: string | null
  updated_at: string | null
  /** Only on the author's own listing — `/api/articles?author=<them>` and
   *  `/api/articles/mine`. Absent everywhere else, and `null` there means the
   *  article is live. */
  taken_down_at?: string | null
  taken_down_reason?: string | null
}

/** The envelope every rust listing answers with. */
export interface ApiPage<T> {
  items: T[]
  page: number
  per_page: number
  total: number
}
