import { APIProvider, App, ThemeProvider } from '@colette/app'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'
import ReactDOM from 'react-dom/client'
import '@colette/react-ui/index.css'
import { CommandsAPI } from './api'

const queryClient = new QueryClient()
const api = new CommandsAPI()

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
