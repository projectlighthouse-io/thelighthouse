import { marked } from 'marked'

/**
 * Splits a lesson's markdown at the paywall marker and renders the chosen half.
 *
 * Server-only, deliberately. The client is never handed both halves and asked
 * to pick — the backend decides free or paid and sends exactly one, so a
 * frontend bug cannot reveal a paid body. Rust takes this over in phase 4; see
 * docs/rebuild.md.
 */
export interface RenderedLesson {
  html: string
  toc: { id: string, text: string }[]
  remainingSections: number
  readMinutes: number
}

export const slugifyHeading = (s: string): string =>
  s
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .trim()
    .replace(/\s+/g, '-')

const headingsIn = (md: string): string[] =>
  md
    .split('\n')
    .filter(l => l.startsWith('## '))
    .map(l => l.replace(/^##\s+/, '').trim())

export function renderLesson(markdown: string, freeSections: number): RenderedLesson {
  // ponytail: the real splitter keys off an explicit marker in frontmatter.
  // here we cut at the Nth h2 so the paywall boundary is visible.
  const marks = [...markdown.matchAll(/\n## /g)].map(m => m.index ?? 0)
  const cut = marks[freeSections] ?? markdown.length

  const free = markdown.slice(0, cut)
  const paid = markdown.slice(cut)

  const words = free.split(/\s+/).length

  return {
    html: marked.parse(free, { async: false }) as string,
    toc: headingsIn(free).map(text => ({ id: slugifyHeading(text), text })),
    remainingSections: headingsIn(paid).length,
    readMinutes: Math.max(1, Math.round(words / 220)),
  }
}
