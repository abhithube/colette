import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import { API } from '@colette/openapi'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider, createRouter } from '@tanstack/react-router'
import { ThemeProvider } from './components/theme-provider'
import { routeTree } from './routeTree.gen'

const queryClient = new QueryClient()
const api = new API({
	baseUrl: import.meta.env.DEV ? import.meta.env.VITE_BACKEND_URL : '',
	credentials: 'include',
})

const router = createRouter({
	routeTree,
	context: {
		queryClient,
		api,
	},
})

declare module '@tanstack/react-router' {
	interface Register {
		router: typeof router
	}
}

ReactDOM.createRoot(document.getElementById('root')!).render(
	<React.StrictMode>
		<ThemeProvider>
			<QueryClientProvider client={queryClient}>
				<RouterProvider router={router} />
			</QueryClientProvider>
		</ThemeProvider>
	</React.StrictMode>,
)
