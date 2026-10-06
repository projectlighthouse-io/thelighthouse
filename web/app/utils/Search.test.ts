import { describe, expect, it } from 'vitest'

import type { SearchEntry } from '@/types/Content'
import { groupEntries, matchEntries } from './Search'

function entry(kind: SearchEntry['kind'], title: string, group: string): SearchEntry {
  return { kind, title, description: '', group, to: `/${title}` }
}

const entries = [
  entry('book', 'C Programming', 'books'),
  entry('chapter', 'Pointers and Memory', 'c programming'),
  entry('lesson', 'Structs and Typedef', 'c programming / pointers and memory'),
  entry('book', 'Containers', 'books'),
  entry('lesson', 'Makefiles', 'c programming / the compilation model'),
  entry('setting', 'Billing and plan', 'settings'),
]

const titles = (list: SearchEntry[]): string[] => list.map(e => e.title)

describe('matchEntries', () => {
  it('returns everything for an empty query', () => {
    expect(matchEntries(entries, '  ', 'all')).toHaveLength(entries.length)
  })

  it('needs every word, in the title or the heading, in any case', () => {
    expect(titles(matchEntries(entries, 'STRUCTS', 'all'))).toEqual(['Structs and Typedef'])
    // `pointers` is in the lesson's heading, not its title.
    expect(titles(matchEntries(entries, 'typedef pointers', 'all'))).toEqual(['Structs and Typedef'])
    expect(matchEntries(entries, 'typedef rust', 'all')).toEqual([])
  })

  it('finds a book\'s lessons by the book\'s name', () => {
    expect(titles(matchEntries(entries, 'c programming', 'lesson'))).toEqual(['Structs and Typedef', 'Makefiles'])
  })

  it('narrows to the chosen kind', () => {
    expect(titles(matchEntries(entries, '', 'book'))).toEqual(['C Programming', 'Containers'])
    expect(titles(matchEntries(entries, '', 'setting'))).toEqual(['Billing and plan'])
  })
})

describe('groupEntries', () => {
  it('gathers rows under headings in order of first appearance', () => {
    const groups = groupEntries(entries)

    expect(groups.map(g => g.label)).toEqual([
      'books',
      'c programming',
      'c programming / pointers and memory',
      'c programming / the compilation model',
      'settings',
    ])
    expect(groups[0]?.items.map(i => i.entry.title)).toEqual(['C Programming', 'Containers'])
  })

  it('numbers rows in the order they are shown, not the order they came in', () => {
    const order = groupEntries(entries).flatMap(g => g.items.map(i => i.at))

    expect(order).toEqual([0, 1, 2, 3, 4, 5])
  })
})
