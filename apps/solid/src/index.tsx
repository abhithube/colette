/* @refresh reload */
import { HttpAPI } from '@colette/core'
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query'
import { render } from 'solid-js/web'
import App from './App'
import './index.css'
import {
  ColorModeProvider,
  ColorModeScript,
  createLocalStorageManager,
} from '@kobalte/core'
import { APIProvider } from './lib/api-context'

const root = document.getElementById('root')

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
  )
}

render(() => {
  const storageManager = createLocalStorageManager('vite-ui-theme')

  return (
    <>
      <ColorModeScript storageType={storageManager.type} />
      <ColorModeProvider
        storageManager={storageManager}
        initialColorMode="system"
      >
        <QueryClientProvider client={new QueryClient()}>
          <APIProvider
            api={
              new HttpAPI({
                baseUrl: import.meta.env.DEV
                  ? import.meta.env.VITE_BACKEND_URL
                  : '/api/v1',
                credentials: 'include',
              })
            }
          >
            <App />
          </APIProvider>
        </QueryClientProvider>
      </ColorModeProvider>
    </>
  )
}, root!)
