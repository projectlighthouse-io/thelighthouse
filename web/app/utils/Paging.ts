/**
 * A paging query parameter as a number, clamped, or `null` when there is
 * nothing usable in it.
 *
 * Rust clamps paging itself, but it clamps *numbers* — `?page=3'` is a serde
 * failure and a 400 before any of that runs. Taking the digits here makes a
 * mangled link a first page rather than an error page. `null` means "not
 * asked for", so the caller can leave the parameter off and let the api pick.
 */
export function numberParam(value: unknown, min: number, max: number): number | null {
  const parsed = Number.parseInt(String(value ?? ''), 10)

  return Number.isNaN(parsed) ? null : Math.min(max, Math.max(min, parsed))
}

/** How many pages a total covers, never fewer than one. */
export function pageCountOf(total: number, perPage: number): number {
  return Math.max(1, Math.ceil(total / Math.max(1, perPage)))
}
