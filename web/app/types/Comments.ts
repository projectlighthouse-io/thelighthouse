/**
 * A lesson's public thread, as `GET /api/books/{book}/lessons/{lesson}/comments`
 * sends it once `useComments` has camel-cased it.
 *
 * Not a `Note`: that is a reader's own row, with privacy and an owner implied
 * by the session. A comment is anybody's, so it carries its author — and only
 * four fields of them, because the api publishes this to anyone who asks.
 */

/** Who wrote a comment. */
export interface CommentAuthor {
  /** `users.id`. Compared with `Reader.sub` (a string) to find the reader's
   *  own comments — `String(id) === sub`, never the other way round. */
  id: number
  /** `null` only for a note whose user row is gone. */
  name: string | null
  username: string | null
  avatarUrl: string | null
}

/** One comment, root or reply alike. */
export interface CommentEntry {
  id: number
  selectedText: string | null
  /** Plain text. Never rendered as html. */
  body: string | null
  startOffset: number | null
  endOffset: number | null
  /** ISO-8601, UTC. `null` for rows migrated without a timestamp. */
  createdAt: string | null
  author: CommentAuthor
}

/** A top-level comment and its replies, oldest reply first. One level only. */
export interface LessonComment extends CommentEntry {
  replies: CommentEntry[]
}
