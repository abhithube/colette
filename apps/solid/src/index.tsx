/* @refresh reload */
import App from './App'
import './index.css'
import { APIProvider } from './lib/api-context'
import { ThemeProvider } from './lib/theme-context'
import { HttpAPI } from '@colette/core'
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query'
import { render } from 'solid-js/web'

const root = document.getElementById('root')

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
  )
}

const api = new HttpAPI({
  baseUrl: import.meta.env.DEV ? import.meta.env.VITE_BACKEND_URL : '/api/v1',
  credentials: 'include',
})
const client = new QueryClient()

render(() => {
  return (
    <>
      <ThemeProvider>
        <QueryClientProvider client={client}>
          <APIProvider api={api}>
            <App />
          </APIProvider>
        </QueryClientProvider>
      </ThemeProvider>
    </>
  )
}, root!)
