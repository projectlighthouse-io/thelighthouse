/**
 * A reader's own articles, according to rust.
 *
 * The mirror of `useNotes`, and the same shape for the same reasons: writes
 * carry the CSRF header, a refusal comes back as a message rather than a
 * thrown error, and what lands in state is the row as stored rather than what
 * was typed.
 *
 * **Client-side only.** `GET /api/articles/mine` is scoped to the session
 * cookie and answered `no-store`; the public listing and the article page are
 * SSR'd through `/_api/blog` instead, because those are the same bytes for
 * everybody and the edge is allowed to hold them.
 *
 * **This never renders markdown.** The body travels as the author typed it and
 * is rendered on the server by `SafeMarkdown`, which sanitises. A component
 * that wants to show somebody else's article asks `/_api/blog/{slug}` for the
 * html; nothing here goes near `v-html`. The editor shows `body` in a
 * `<textarea>`, which is text and cannot execute.
 */

import { csrfHeader } from '@/composables/UseReader'

/** The topics rust accepts, in the order the picker shows them. Mirrors
 *  `articles::payload::CATEGORIES`; rust refuses anything not on its own list,
 *  so a drift here is a refused write rather than a bad row. */
export const CATEGORIES = [
  'go',
  'rust',
  'containers',
  'systems',
  'docker',
  'kubernetes',
  'networking',
  'dsa',
] as const

export type Category = typeof CATEGORIES[number]

/** Mirrors `articles::payload::MAX_TOPICS`. Rust refuses more. */
export const MAX_TOPICS = 3

/** One of the reader's own articles, taken down ones included. */
export interface OwnArticle {
  id: number
  slug: string
  title: string
  /** The line under the title. Author-written and required. */
  subtitle: string
  /** What the article is about. One at least, `MAX_TOPICS` at most. */
  topics: string[]
  /** Markdown, as typed. Rendered only by the server. */
  body: string
  createdAt: string | null
  updatedAt: string | null
  /** Set when an admin has taken it down; `null` while it is live. */
  takenDownAt: string | null
  /** Why, shown to its author and to nobody else. */
  takenDownReason: string | null
}

/** The wire shape, which is snake_case because the columns are. */
interface OwnArticleResponse {
  id: number
  slug: string
  title: string
  subtitle: string
  topics: string[]
  body: string
  created_at: string | null
  updated_at: string | null
  taken_down_at: string | null
  taken_down_reason: string | null
}

interface PageResponse<T> {
  items: T[]
  page: number
  per_page: number
  total: number
}

/** Shown when the api refused but said nothing a reader can act on. */
const GENERIC_FAILURE = 'That could not be saved. Please try again.'

function toArticle(article: OwnArticleResponse): OwnArticle {
  return {
    id: article.id,
    slug: article.slug,
    title: article.title,
    subtitle: article.subtitle,
    topics: article.topics,
    body: article.body,
    createdAt: article.created_at,
    updatedAt: article.updated_at,
    takenDownAt: article.taken_down_at,
    takenDownReason: article.taken_down_reason,
  }
}

export function useArticles() {
  const articles = ref<OwnArticle[]>([])
  const total = ref<number>(0)

  // Distinct from `articles.length === 0`, which cannot tell "nothing written
  // yet" from "not asked yet" — and the difference is an empty state shown to
  // somebody who has articles, for as long as the request takes.
  const loaded = ref<boolean>(false)
  const pending = ref<boolean>(false)

  /**
   * The reader's own articles, for the editor to find one in.
   *
   * The listing that used to read this is gone: `/blog?author=` is where an
   * author reads their own shelf now, and rust decides what that contains —
   * see `articles::handler::shelf`.
   */
  async function load(): Promise<void> {
    if (import.meta.server) return

    pending.value = true

    try {
      const response = await $fetch<PageResponse<OwnArticleResponse>>(
        '/api/articles/mine',
        { query: { per_page: 25 } },
      )

      articles.value = response.items.map(toArticle)
      total.value = response.total
    }
    catch {
      // Including a 401: a signed-out reader has nothing of their own to show.
      articles.value = []
      total.value = 0
    }
    finally {
      pending.value = false
      loaded.value = true
    }
  }

  /** One of the reader's own, for the editor to open. */
  async function find(slug: string): Promise<OwnArticle | null> {
    if (!loaded.value) await load()

    return articles.value.find(article => article.slug === slug) ?? null
  }

  /**
   * Publishes an article. Live immediately — rust has no pending state.
   *
   * Returns the saved article on success and the refusal otherwise, so a caller
   * can navigate to the slug rust minted rather than guessing at one.
   */
  async function create(
    title: string,
    subtitle: string,
    topics: string[],
    body: string,
  ): Promise<{ article: OwnArticle } | { refused: Refused }> {
    try {
      const saved = await $fetch<OwnArticleResponse>('/api/articles', {
        method: 'POST',
        headers: csrfHeader(),
        body: { title, subtitle, topics, body },
      })

      const article = toArticle(saved)
      articles.value.unshift(article)
      total.value += 1

      return { article }
    }
    catch (error) {
      return { refused: refusedBy(error, GENERIC_FAILURE) }
    }
  }

  /**
   * Rewrites one of the reader's own.
   *
   * Addressed by slug, which is how rust mounts it — and the slug does not
   * move when the title does, so a link somebody shared keeps working.
   */
  async function edit(
    slug: string,
    title: string,
    subtitle: string,
    topics: string[],
    body: string,
  ): Promise<{ article: OwnArticle } | { refused: Refused }> {
    try {
      const updated = await $fetch<OwnArticleResponse>(
        `/api/articles/${encodeURIComponent(slug)}`,
        {
          method: 'PATCH',
          headers: csrfHeader(),
          body: { title, subtitle, topics, body },
        },
      )

      const article = toArticle(updated)
      const at = articles.value.findIndex(it => it.slug === slug)
      if (at !== -1) articles.value[at] = article

      return { article }
    }
    catch (error) {
      return { refused: refusedBy(error, GENERIC_FAILURE) }
    }
  }

  async function remove(slug: string): Promise<Refused | null> {
    try {
      await $fetch(`/api/articles/${encodeURIComponent(slug)}`, {
        method: 'DELETE',
        headers: csrfHeader(),
      })

      articles.value = articles.value.filter(it => it.slug !== slug)
      total.value = Math.max(0, total.value - 1)

      return null
    }
    catch (error) {
      return refusedBy(error, GENERIC_FAILURE)
    }
  }

  return { articles, total, loaded, pending, load, find, create, edit, remove }
}
