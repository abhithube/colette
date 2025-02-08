import { App } from './app'
import { ThemeProvider } from './components/theme-provider'
import './index.css'
import { APIProvider } from './lib/api-context'
import { HttpAPI } from '@colette/core'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'
import ReactDOM from 'react-dom/client'

const queryClient = new QueryClient()
const api = new HttpAPI({
  baseUrl: import.meta.env.DEV ? import.meta.env.VITE_BACKEND_URL : '/api/v1',
  credentials: 'include',
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <APIProvider api={api}>
      <ThemeProvider>
        <QueryClientProvider client={queryClient}>
          <App />
        </QueryClientProvider>
      </ThemeProvider>
    </APIProvider>
  </React.StrictMode>,
)
