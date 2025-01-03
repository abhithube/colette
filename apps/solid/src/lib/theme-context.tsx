import {
  type ParentComponent,
  createContext,
  createEffect,
  createSignal,
  mergeProps,
  splitProps,
  useContext,
} from 'solid-js'

type Theme = 'dark' | 'light' | 'system'

type ThemeState = {
  theme: Theme
  setTheme: (theme: Theme) => void
}

const ThemeContext = createContext<ThemeState>({
  theme: 'system',
  setTheme: () => null,
})

type ThemeProviderProps = {
  defaultTheme?: Theme
  storageKey?: string
}

export const ThemeProvider: ParentComponent<ThemeProviderProps> = (
  rawProps,
) => {
  const props = mergeProps(
    {
      defaultTheme: 'system',
      storageKey: 'vite-ui-theme',
    } as { defaultTheme: Theme; storageKey: string },
    rawProps,
  )
  const [local, others] = splitProps(props, [
    'defaultTheme',
    'storageKey',
    'children',
  ])

  const [theme, setTheme] = createSignal<Theme>(
    (localStorage.getItem(local.storageKey) as Theme) || local.defaultTheme,
  )

  createEffect(() => {
    const root = window.document.documentElement

    root.classList.remove('light', 'dark')

    if (theme() === 'system') {
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)')
        .matches
        ? 'dark'
        : 'light'

      root.classList.add(systemTheme)
      return
    }

    root.classList.add(theme())
  }, [theme])

  return (
    <ThemeContext.Provider
      {...others}
      value={{
        theme: theme(),
        setTheme: (theme: Theme) => {
          localStorage.setItem(local.storageKey, theme)
          setTheme(theme)
        },
      }}
    >
      {local.children}
    </ThemeContext.Provider>
  )
}

export const useTheme = () => {
  const context = useContext(ThemeContext)

  if (context === undefined)
    throw new Error('useTheme must be used within a ThemeProvider')

  return context
}
