import { describe, expect, it } from 'vitest'

import { lessonTitle, metaDescription, pageTitle } from './Seo'

describe('pageTitle', () => {
  it('names the site after a title that does not', () => {
    expect(pageTitle('Programming Books', 'projectlighthouse')).toBe('Programming Books — projectlighthouse')
  })

  it('leaves a title that already names the site alone', () => {
    expect(pageTitle('Pricing — projectlighthouse', 'projectlighthouse')).toBe('Pricing — projectlighthouse')
  })
})

describe('metaDescription', () => {
  it('leaves a description that fits alone', () => {
    expect(metaDescription('Short and complete.')).toBe('Short and complete.')
  })

  it('cuts a long one at a word, under the limit, and says it was cut', () => {
    const long = 'word '.repeat(60).trim()
    const cut = metaDescription(long)

    expect(cut.length).toBeLessThanOrEqual(160)
    expect(cut.endsWith('…')).toBe(true)
    expect(cut.slice(0, -1).endsWith('word')).toBe(true)
  })

  it('does not leave a dangling comma or dash before the ellipsis', () => {
    const cut = metaDescription(`${'a '.repeat(77)}bb, cc dd ee ff gg hh`)

    expect(cut).not.toMatch(/[,—-]…$/)
  })

  it('collapses whitespace from multi-line copy', () => {
    expect(metaDescription('one\n  two\tthree')).toBe('one two three')
  })
})

describe('lessonTitle', () => {
  it('names the book while the whole title fits a result', () => {
    expect(lessonTitle('Hello World', 'C Programming')).toBe('Hello World — C Programming')
  })

  it('drops the book once lesson, book and site no longer fit', () => {
    expect(lessonTitle('There Is No Such Thing as a Container', 'Build Your Own Container'))
      .toBe('There Is No Such Thing as a Container')
  })
})
