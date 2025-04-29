import './index.css'
import { Layout } from './layout'
import { StashPage } from './routes/bookmarks'
import { CollectionPage } from './routes/collections/id'
import { HomePage } from './routes/home'
import { LoginPage } from './routes/login'
import { RegisterPage } from './routes/register'
import { StreamPage } from './routes/streams/id'
import { SubscriptionsPage } from './routes/subscriptions'
import { SubscriptionPage } from './routes/subscriptions/id'
import { HttpAPI } from '@colette/core'
import { buildRouter } from '@colette/router'
import { ThemeProvider } from '@colette/util'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Outlet, RouteIds, RouterProvider } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools'
import React, { JSX } from 'react'
import ReactDOM from 'react-dom/client'

const queryClient = new QueryClient()
const api = new HttpAPI({
  baseUrl: import.meta.env.DEV ? import.meta.env.VITE_BACKEND_URL : '/api',
  credentials: 'include',
})

const router = buildRouter(api, queryClient)

type RouterIds = RouteIds<(typeof router)['routeTree']>

const routerMap = {
  __root__: () => (
    <>
      <Outlet />
      <TanStackRouterDevtools />
      <ReactQueryDevtools />
    </>
  ),
  '/register': RegisterPage,
  '/login': LoginPage,
  '/layout': Layout,
  '/layout/': HomePage,
  '/layout/subscriptions': SubscriptionsPage,
  '/layout/subscriptions/': () => <div></div>,
  '/layout/subscriptions/$subscriptionId': SubscriptionPage,
  '/layout/stash': StashPage,
  '/layout/streams': () => <div></div>,
  '/layout/streams/': () => <div></div>,
  '/layout/streams/$streamId': StreamPage,
  '/layout/collections': () => <div></div>,
  '/layout/collections/': () => <div></div>,
  '/layout/collections/$collectionId': CollectionPage,
} as const satisfies Record<RouterIds, () => JSX.Element | undefined>

Object.entries(routerMap).forEach(([path, component]) => {
  const foundRoute = router.routesById[path as RouterIds]
  foundRoute.update({
    component,
  })
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </ThemeProvider>
  </React.StrictMode>,
)
