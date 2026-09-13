/**
 * The one rule two routes answer with.
 *
 * It lived in both of them and drifted — the contents list kept only the lesson
 * half and went on calling lessons paid to readers who own them. These are the
 * four combinations, so a future simplification has to disagree with one of
 * them out loud.
 */

import { describe, expect, it } from 'vitest'
import { isLocked } from './Lighthouse'

describe('isLocked', () => {
  it('is locked when something is withheld and the reader may not see it', () => {
    expect(isLocked({ has_paid_part: true, unlocked: false })).toBe(true)
  })

  it('is open once the reader may see the withheld part', () => {
    expect(isLocked({ has_paid_part: true, unlocked: true })).toBe(false)
  })

  it('is open when the lesson withholds nothing, whoever is asking', () => {
    expect(isLocked({ has_paid_part: false, unlocked: false })).toBe(false)
    expect(isLocked({ has_paid_part: false, unlocked: true })).toBe(false)
  })

  it('treats an absent reader half as not unlocked', () => {
    // The shared, edge-cached listing is answered without a session, so there
    // is no reader half to have. Locked is the honest answer for a response
    // that cannot know; the page asks again from the browser to soften it.
    expect(isLocked({ has_paid_part: true })).toBe(true)
    expect(isLocked({ has_paid_part: false })).toBe(false)
  })
})
