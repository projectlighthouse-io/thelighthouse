/**
 * What the api says when it refuses a write, in the one shape every form reads.
 *
 * Rust answers a refused field with `{ code: "invalid", error, fields }`, one
 * message per field the form drew; anything else — a parent note that is gone,
 * a 429, the network — with at most `{ code, error }`. Both land here, so a
 * form never parses an error body itself.
 */
export interface Refused {
  /** For the form as a whole. Empty when every problem has a field. */
  message: string
  /** Keyed by the field name the request sent, as the api echoes it. */
  fields: Record<string, string>
}

/**
 * The api's refusal, or `fallback` when it said nothing a reader can act on.
 *
 * The api's messages are written for the person who typed the form, so they
 * are shown as given. A body with field errors carries its `error` too, but
 * that line only restates "some fields need another look" — the fields say
 * which, so it is dropped rather than shown twice.
 */
export function refusedBy(error: unknown, fallback: string): Refused {
  const data = (error as { data?: { error?: unknown, fields?: unknown } } | undefined)?.data
  const fields = isFieldMap(data?.fields) ? data.fields : {}

  if (Object.keys(fields).length > 0) return { message: '', fields }

  return {
    message: typeof data?.error === 'string' ? data.error : fallback,
    fields: {},
  }
}

function isFieldMap(value: unknown): value is Record<string, string> {
  return typeof value === 'object'
    && value !== null
    && Object.values(value).every(v => typeof v === 'string')
}
