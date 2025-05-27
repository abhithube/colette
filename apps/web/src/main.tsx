import './index.css'
import {
  CollectionPage,
  HomePage,
  Layout,
  LoginPage,
  StashPage,
  StreamPage,
  SubscriptionPage,
  SubscriptionsPage,
} from '@colette/app'
import { client } from '@colette/core'
import {
  buildRouter,
  getRouteApi,
  Outlet,
  RouteIds,
  RouterProvider,
} from '@colette/router'
import { OIDCConfigProvider, ThemeProvider } from '@colette/util'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools'
import React, { JSX } from 'react'
import ReactDOM from 'react-dom/client'

client.setConfig({
  baseURL: import.meta.env.DEV ? import.meta.env.VITE_BACKEND_URL : '/api',
})

const queryClient = new QueryClient()

const router = buildRouter(queryClient)

type RouterIds = RouteIds<(typeof router)['routeTree']>

const routerMap = {
  __root__: () => {
    const context = getRouteApi('__root__').useRouteContext()

    return (
      <OIDCConfigProvider oidcConfig={context.oidcConfig}>
        <ThemeProvider>
          <QueryClientProvider client={context.queryClient}>
            <Outlet />
            <TanStackRouterDevtools />
            <ReactQueryDevtools />
          </QueryClientProvider>
        </ThemeProvider>
      </OIDCConfigProvider>
    )
  },
  '/login': LoginPage,
  '/auth-callback': () => <div></div>,
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
    <RouterProvider router={router} />
  </React.StrictMode>,
)
