import type { SearchEntry } from '@/types/Content'

/** What the palette's tabs narrow to. */
export type SearchFilter = 'all' | SearchEntry['kind']

/** Every word typed has to appear in the title or the heading it sits under. */
export function matchEntries(entries: SearchEntry[], query: string, filter: SearchFilter): SearchEntry[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean)

  return entries.filter((entry) => {
    if (filter !== 'all' && entry.kind !== filter) return false

    const text = `${entry.title} ${entry.group}`.toLowerCase()

    return words.every(word => text.includes(word))
  })
}

export interface SearchGroup {
  label: string
  items: { entry: SearchEntry, at: number }[]
}

/**
 * Under their headings, in order of first appearance. `at` is the row's place
 * reading down the list, which is what the arrow keys walk — not its place in
 * the input, which a heading gathering rows from further on reorders.
 */
export function groupEntries(entries: SearchEntry[]): SearchGroup[] {
  const byLabel = new Map<string, SearchEntry[]>()

  for (const entry of entries) {
    byLabel.set(entry.group, [...(byLabel.get(entry.group) ?? []), entry])
  }

  let at = 0

  return [...byLabel].map(([label, grouped]) => ({
    label,
    items: grouped.map(entry => ({ entry, at: at++ })),
  }))
}
