import { afterEach, beforeEach, describe, expect, test } from 'vitest'
import { NucleoMatcher } from '../dist/nucleo_wasm.js'

const FRUITS = ['apple', 'apricot', 'banana', 'blueberry', 'cherry', 'grape', 'grapefruit']
const PATHS = ['src/components/Button.tsx', 'src/utils/helpers.ts', 'tests/index.test.ts']

describe('NucleoMatcher', () => {
  describe('constructor', () => {
    test('creates a matcher with default options', () => {
      const matcher = new NucleoMatcher(FRUITS)
      expect(matcher).toBeInstanceOf(NucleoMatcher)
      matcher.free()
    })

    test('accepts an empty item list', () => {
      const matcher = new NucleoMatcher([])
      expect(matcher).toBeInstanceOf(NucleoMatcher)
      matcher.free()
    })

    test('accepts all MatcherOptions', () => {
      const matcher = new NucleoMatcher(FRUITS, {
        matchPaths: true,
        preferPrefix: true,
        caseMatching: 'respect',
        normalization: 'never',
      })
      expect(matcher).toBeInstanceOf(NucleoMatcher)
      matcher.free()
    })
  })

  describe('matchLiteral', () => {
    let matcher: NucleoMatcher

    beforeEach(() => {
      matcher = new NucleoMatcher(FRUITS)
    })

    afterEach(() => {
      matcher.free()
    })

    test('returns matches for fuzzy kind (default)', () => {
      const results = matcher.matchLiteral('ap') as [string, number][]
      expect(results.length).toBeGreaterThan(0)
      const items = results.map(([item]) => item)
      expect(items).toContain('apple')
      expect(items).toContain('apricot')
      expect(items).toContain('grape')
      expect(items).toContain('grapefruit')
    })

    test('fuzzy kind returns [item, score] tuples', () => {
      const results = matcher.matchLiteral('apple') as [string, number][]
      expect(results.length).toBeGreaterThan(0)
      const [item, score] = results[0]
      expect(typeof item).toBe('string')
      expect(typeof score).toBe('number')
    })

    test('substring kind only returns items containing the substring', () => {
      const results = matcher.matchLiteral('erry', 'substring') as [string, number][]
      const items = results.map(([item]) => item)
      expect(items).toContain('blueberry')
      expect(items).toContain('cherry')
      expect(items).not.toContain('apple')
      expect(items).not.toContain('grape')
    })

    test('prefix kind only returns items starting with pattern', () => {
      const results = matcher.matchLiteral('ap', 'prefix') as [string, number][]
      const items = results.map(([item]) => item)
      expect(items).toContain('apple')
      expect(items).toContain('apricot')
      expect(items).not.toContain('grape')
      expect(items).not.toContain('grapefruit')
    })

    test('postfix kind only returns items ending with pattern', () => {
      const results = matcher.matchLiteral('ry', 'postfix') as [string, number][]
      const items = results.map(([item]) => item)
      expect(items).toContain('cherry')
      expect(items).toContain('blueberry')
      expect(items).not.toContain('apple')
    })

    test('exact kind only returns exact matches', () => {
      const results = matcher.matchLiteral('apple', 'exact') as [string, number][]
      const items = results.map(([item]) => item)
      expect(items).toContain('apple')
      expect(items).not.toContain('apricot')
      expect(items).not.toContain('grapefruit')
    })

    test('returns empty array when nothing matches', () => {
      const results = matcher.matchLiteral('zzz') as [string, number][]
      expect(results).toEqual([])
    })

    test('exact kind returns empty array for non-existent item', () => {
      const results = matcher.matchLiteral('pineapple', 'exact') as [string, number][]
      expect(results).toEqual([])
    })

    test('results are sorted by score descending', () => {
      const results = matcher.matchLiteral('grape') as [string, number][]
      const scores = results.map(([, score]) => score)
      for (let i = 1; i < scores.length; i++) {
        expect(scores[i - 1]).toBeGreaterThanOrEqual(scores[i])
      }
    })
  })

  describe('matchPattern', () => {
    let matcher: NucleoMatcher

    beforeEach(() => {
      matcher = new NucleoMatcher(FRUITS)
    })

    afterEach(() => {
      matcher.free()
    })

    test('basic fuzzy match returns [item, score] pairs', () => {
      const results = matcher.matchPattern('ap') as [string, number][]
      expect(results.length).toBeGreaterThan(0)
      const [item, score] = results[0]
      expect(typeof item).toBe('string')
      expect(typeof score).toBe('number')
    })

    test('^ prefix syntax matches only items starting with pattern', () => {
      const results = matcher.matchPattern('^ap') as [string, number][]
      const items = results.map(([item]) => item)
      expect(items).toContain('apple')
      expect(items).toContain('apricot')
      expect(items).not.toContain('grape')
    })

    test('$ postfix syntax matches only items ending with pattern', () => {
      const results = matcher.matchPattern('ry$') as [string, number][]
      const items = results.map(([item]) => item)
      expect(items).toContain('cherry')
      expect(items).toContain('blueberry')
      expect(items).not.toContain('apple')
    })

    test("' quote forces substring match", () => {
      const results = matcher.matchPattern("'erry") as [string, number][]
      const items = results.map(([item]) => item)
      expect(items).toContain('blueberry')
      expect(items).toContain('cherry')
      expect(items).not.toContain('apple')
    })

    test('! negation excludes matching items', () => {
      const all = matcher.matchPattern('') as [string, number][]
      const negated = matcher.matchPattern('!apple') as [string, number][]
      const negatedItems = negated.map(([item]) => item)
      expect(negatedItems).not.toContain('apple')
      expect(negated.length).toBeLessThan(all.length)
    })

    test('returns empty array when nothing matches', () => {
      const results = matcher.matchPattern('^zzz') as [string, number][]
      expect(results).toEqual([])
    })

    test('results are sorted by score descending', () => {
      const results = matcher.matchPattern('ap') as [string, number][]
      const scores = results.map(([, score]) => score)
      for (let i = 1; i < scores.length; i++) {
        expect(scores[i - 1]).toBeGreaterThanOrEqual(scores[i])
      }
    })
  })

  describe('matchLiteralIndices', () => {
    let matcher: NucleoMatcher

    beforeEach(() => {
      matcher = new NucleoMatcher(FRUITS)
    })

    afterEach(() => {
      matcher.free()
    })

    test('returns [item, score, indices] triples', () => {
      const results = matcher.matchLiteralIndices('apple', 'exact') as [string, number, number[]][]
      expect(results.length).toBeGreaterThan(0)
      const [item, score, indices] = results[0]
      expect(typeof item).toBe('string')
      expect(typeof score).toBe('number')
      expect(Array.isArray(indices)).toBe(true)
    })

    test('indices are within bounds of the matched item', () => {
      const results = matcher.matchLiteralIndices('ap') as [string, number, number[]][]
      for (const [item, , indices] of results) {
        for (const idx of indices) {
          expect(idx).toBeGreaterThanOrEqual(0)
          expect(idx).toBeLessThan(item.length)
        }
      }
    })

    test('returns same items and scores as matchLiteral', () => {
      const plain = matcher.matchLiteral('ap') as [string, number][]
      const withIndices = matcher.matchLiteralIndices('ap') as [string, number, number[]][]
      expect(withIndices.map(([i, s]) => [i, s])).toEqual(plain)
    })

    test('returns empty array when nothing matches', () => {
      const results = matcher.matchLiteralIndices('zzz')
      expect(results).toEqual([])
    })
  })

  describe('matchPatternIndices', () => {
    let matcher: NucleoMatcher

    beforeEach(() => {
      matcher = new NucleoMatcher(FRUITS)
    })

    afterEach(() => {
      matcher.free()
    })

    test('returns [item, score, indices] triples', () => {
      const results = matcher.matchPatternIndices('apple') as [string, number, number[]][]
      expect(results.length).toBeGreaterThan(0)
      const [item, score, indices] = results[0]
      expect(typeof item).toBe('string')
      expect(typeof score).toBe('number')
      expect(Array.isArray(indices)).toBe(true)
    })

    test('indices are within bounds of the matched item', () => {
      const results = matcher.matchPatternIndices('ap') as [string, number, number[]][]
      for (const [item, , indices] of results) {
        for (const idx of indices) {
          expect(idx).toBeGreaterThanOrEqual(0)
          expect(idx).toBeLessThan(item.length)
        }
      }
    })

    test('returns same items and scores as matchPattern', () => {
      const plain = matcher.matchPattern('ap') as [string, number][]
      const withIndices = matcher.matchPatternIndices('ap') as [string, number, number[]][]
      expect(withIndices.map(([i, s]) => [i, s])).toEqual(plain)
    })

    test('returns empty array when nothing matches', () => {
      const results = matcher.matchPatternIndices('^zzz')
      expect(results).toEqual([])
    })
  })

  describe('score', () => {
    let matcher: NucleoMatcher

    beforeEach(() => {
      matcher = new NucleoMatcher([])
    })

    afterEach(() => {
      matcher.free()
    })

    test('returns a number for a matching haystack', () => {
      const result = matcher.score('apple', 'apple')
      expect(typeof result).toBe('number')
    })

    test('returns undefined for a non-matching haystack', () => {
      const result = matcher.score('^zzz', 'apple')
      expect(result).toBeUndefined()
    })

    test('exact match scores higher than fuzzy partial match', () => {
      const exact = matcher.score('apple', 'apple') as number
      const partial = matcher.score('apple', 'pineapple') as number
      expect(exact).toBeGreaterThan(partial)
    })

    test('works independently of stored items', () => {
      const matcherWithItems = new NucleoMatcher(FRUITS)
      const scoreA = matcher.score('grape', 'grape') as number
      const scoreB = matcherWithItems.score('grape', 'grape') as number
      expect(scoreA).toBe(scoreB)
      matcherWithItems.free()
    })
  })

  describe('setItems', () => {
    let matcher: NucleoMatcher

    beforeEach(() => {
      matcher = new NucleoMatcher(FRUITS)
    })

    afterEach(() => {
      matcher.free()
    })

    test('replaces stored items', () => {
      matcher.setItems(['mango', 'melon', 'papaya'])
      const results = matcher.matchLiteral('mango', 'exact') as [string, number][]
      expect(results.map(([item]) => item)).toContain('mango')
    })

    test('old items are no longer returned after setItems', () => {
      matcher.setItems(['mango', 'melon', 'papaya'])
      const results = matcher.matchLiteral('apple', 'exact') as [string, number][]
      expect(results).toEqual([])
    })

    test('can set an empty list', () => {
      matcher.setItems([])
      const results = matcher.matchLiteral('apple') as [string, number][]
      expect(results).toEqual([])
    })
  })

  describe('caseMatching option', () => {
    test('ignore: matches regardless of case (default)', () => {
      const matcher = new NucleoMatcher(FRUITS, { caseMatching: 'ignore' })
      const lower = matcher.matchLiteral('Apple', 'exact') as [string, number][]
      expect(lower.map(([item]) => item)).toContain('apple')
      matcher.free()
    })

    test('respect: uppercase pattern does not match lowercase item', () => {
      const matcher = new NucleoMatcher(FRUITS, { caseMatching: 'respect' })
      const results = matcher.matchLiteral('Apple', 'exact') as [string, number][]
      expect(results.map(([item]) => item)).not.toContain('apple')
      matcher.free()
    })

    test('respect: lowercase pattern matches lowercase item', () => {
      const matcher = new NucleoMatcher(FRUITS, { caseMatching: 'respect' })
      const results = matcher.matchLiteral('apple', 'exact') as [string, number][]
      expect(results.map(([item]) => item)).toContain('apple')
      matcher.free()
    })

    test('smart: uppercase pattern is case-sensitive', () => {
      const matcher = new NucleoMatcher(FRUITS, { caseMatching: 'smart' })
      const results = matcher.matchLiteral('Apple', 'exact') as [string, number][]
      expect(results.map(([item]) => item)).not.toContain('apple')
      matcher.free()
    })

    test('smart: lowercase pattern is case-insensitive', () => {
      const matcher = new NucleoMatcher(['Apple', 'APPLE', 'apple'], { caseMatching: 'smart' })
      const results = matcher.matchLiteral('apple', 'exact') as [string, number][]
      expect(results.length).toBe(3)
      matcher.free()
    })

    test('per-call caseMatching overrides constructor default', () => {
      const matcher = new NucleoMatcher(FRUITS, { caseMatching: 'respect' })
      const results = matcher.matchLiteral('Apple', 'exact', { caseMatching: 'ignore', normalization: undefined }) as [string, number][]
      expect(results.map(([item]) => item)).toContain('apple')
      matcher.free()
    })
  })

  describe('matchPaths option', () => {
    test('treats path separators as word boundaries', () => {
      const matcher = new NucleoMatcher(PATHS, { matchPaths: true })
      const results = matcher.matchLiteral('Button') as [string, number][]
      const items = results.map(([item]) => item)
      expect(items).toContain('src/components/Button.tsx')
      matcher.free()
    })
  })

  describe('edge cases', () => {
    test('empty items list returns empty results', () => {
      const matcher = new NucleoMatcher([])
      const results = matcher.matchLiteral('apple')
      expect(results).toEqual([])
      matcher.free()
    })

    test('single-character pattern works', () => {
      const matcher = new NucleoMatcher(FRUITS)
      const results = matcher.matchLiteral('a') as [string, number][]
      expect(results.length).toBeGreaterThan(0)
      matcher.free()
    })

    test('handles unicode items', () => {
      const items = ['café', 'naïve', 'résumé', 'plain']
      const matcher = new NucleoMatcher(items)
      const results = matcher.matchLiteral('cafe') as [string, number][]
      expect(results.length).toBeGreaterThan(0)
      matcher.free()
    })

    test('Symbol.dispose frees the matcher', () => {
      expect(() => {
        const matcher = new NucleoMatcher(FRUITS)
        matcher[Symbol.dispose]()
      }).not.toThrow()
    })
  })
})
