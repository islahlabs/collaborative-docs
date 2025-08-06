import { describe, it, expect } from 'vitest'
import { cn } from './utils'

describe('Utils', () => {
  describe('cn function', () => {
    it('should merge class names correctly', () => {
      const result = cn('text-red-500', 'bg-blue-500', 'p-4')
      expect(result).toBe('text-red-500 bg-blue-500 p-4')
    })

    it('should handle conditional classes', () => {
      const isActive = true
      const result = cn(
        'base-class',
        isActive && 'active-class',
        'always-present'
      )
      expect(result).toBe('base-class active-class always-present')
    })

    it('should handle conditional classes with false values', () => {
      const isActive = false
      const result = cn(
        'base-class',
        isActive && 'active-class',
        'always-present'
      )
      expect(result).toBe('base-class always-present')
    })

    it('should deduplicate conflicting Tailwind classes', () => {
      const result = cn('text-red-500', 'text-blue-500', 'p-4')
      // The last conflicting class should win
      expect(result).toBe('text-blue-500 p-4')
    })

    it('should handle undefined and null values', () => {
      const result = cn('base-class', undefined, null, 'valid-class')
      expect(result).toBe('base-class valid-class')
    })

    it('should handle arrays of classes', () => {
      const result = cn(['class1', 'class2'], 'class3', ['class4', 'class5'])
      expect(result).toBe('class1 class2 class3 class4 class5')
    })

    it('should handle objects with boolean values', () => {
      const result = cn({
        'always': true,
        'never': false,
        'conditional': true
      })
      expect(result).toBe('always conditional')
    })
  })
}) 