import { ThemeProvider, createRouter } from '@colette/app'
import { HttpAPI } from '@colette/core'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'

const queryClient = new QueryClient()
const api = new HttpAPI({
  baseUrl: import.meta.env.DEV ? import.meta.env.VITE_BACKEND_URL : '/api/v1',
  credentials: 'include',
})

const router = createRouter(queryClient, api)

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </ThemeProvider>
  </React.StrictMode>,
)
