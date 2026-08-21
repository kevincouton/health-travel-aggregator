import { beforeEach, describe, expect, it } from 'vitest'

import { useTheme } from '../../composables/useTheme'

describe('useTheme', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.classList.remove('dark')
  })

  it('defaults to the system theme when nothing is stored', () => {
    const { theme } = useTheme()
    expect(theme.value).toBe('system')
  })

  it('restores a valid stored theme', () => {
    localStorage.setItem('theme', 'dark')
    const { theme, isDark } = useTheme()
    expect(theme.value).toBe('dark')
    expect(isDark.value).toBe(true)
  })

  it('ignores an invalid stored theme and falls back to system', () => {
    localStorage.setItem('theme', 'neon')
    const { theme } = useTheme()
    expect(theme.value).toBe('system')
  })

  it('setTheme persists the choice and toggles the dark class', () => {
    const { theme, isDark, setTheme } = useTheme()

    setTheme('dark')
    expect(theme.value).toBe('dark')
    expect(isDark.value).toBe(true)
    expect(localStorage.getItem('theme')).toBe('dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)

    setTheme('light')
    expect(isDark.value).toBe(false)
    expect(localStorage.getItem('theme')).toBe('light')
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })

  it('toggleTheme flips between dark and light', () => {
    // jsdom has no matchMedia, so the system theme resolves to light.
    const { isDark, toggleTheme } = useTheme()
    expect(isDark.value).toBe(false)

    toggleTheme()
    expect(isDark.value).toBe(true)
    expect(document.documentElement.classList.contains('dark')).toBe(true)

    toggleTheme()
    expect(isDark.value).toBe(false)
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })
})
