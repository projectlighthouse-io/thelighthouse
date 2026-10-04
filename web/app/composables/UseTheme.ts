export type Theme = 'light' | 'dark'

const STORAGE_KEY = 'theme'

/**
 * Theme lives in localStorage rather than a cookie, unlike locale.
 *
 * Locale changes what the HTML says, so the server has to know it before
 * rendering. Theme changes only which CSS variables apply — the markup is
 * identical either way — so keeping it out of the request means anonymous
 * pages stay one cached artifact instead of two.
 *
 * PARKED until dark mode is designed: the toggle is not rendered and the
 * inline script that applied the stored choice before first paint is gone
 * from nuxt.config. Bringing it back means restoring that script — setting
 * `data-theme` on <html>, which is what the tokens in colors.css key off —
 * and putting ChromeThemeToggle back in SiteHeader.
 */
export function useTheme() {
  // 'light' on the server, corrected on mount where the DOM exists
  const theme = useState<Theme>('theme', () => 'light')

  const isDark = computed<boolean>(() => theme.value === 'dark')

  const commit = (next: Theme): void => {
    theme.value = next
    localStorage.setItem(STORAGE_KEY, next)
    document.documentElement.dataset.theme = next
    document.documentElement.style.colorScheme = next
  }

  const set = async (next: Theme): Promise<void> => {
    const root = document.documentElement
    const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches

    // Without the API, transition.css cross-fades each element's colours
    // instead. Same effect, less smoothly, and nothing breaks.
    if (reduced || !document.startViewTransition) {
      if (!reduced) {
        root.dataset.themeSwap = ''
        window.setTimeout(() => delete root.dataset.themeSwap, 320)
      }
      commit(next)
      return
    }

    root.dataset.themeReveal = ''

    const transition = document.startViewTransition(async () => {
      commit(next)
      // the snapshot is taken from the DOM this callback leaves behind, and
      // vue patches on the next tick — without awaiting it the icon swap is
      // captured in the old frame
      await nextTick()
    })

    try {
      await transition.finished
    }
    finally {
      delete root.dataset.themeReveal
    }
  }

  const toggle = (): Promise<void> => set(isDark.value ? 'light' : 'dark')

  onMounted(() => {
    // read back what the inline script already decided, so the button starts in
    // agreement with the page rather than guessing
    theme.value = document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light'
  })

  return { theme, isDark, set, toggle }
}
