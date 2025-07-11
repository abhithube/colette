import { indexRoute } from './routes/(app)'
import { collectionsRoute } from './routes/(app)/collections'
import { collectionsIdRoute } from './routes/(app)/collections/$id'
import { collectionsIndexRoute } from './routes/(app)/collections/index'
import { layoutRoute } from './routes/(app)/layout'
import { stashRoute } from './routes/(app)/stash'
import { streamsRoute } from './routes/(app)/streams'
import { streamsIdRoute } from './routes/(app)/streams/$id'
import { streamsIndexRoute } from './routes/(app)/streams/index'
import { subscriptionsIdRoute } from './routes/(app)/subscriptions/$id'
import { subscriptionsIndexRoute } from './routes/(app)/subscriptions/index'
import { authCallbackRoute } from './routes/(auth)/auth-callback'
import { loginRoute } from './routes/(auth)/login'
import { registerRoute } from './routes/(auth)/register'
import { rootRoute } from './routes/__root'
import { QueryClient } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'

const routeTree = rootRoute.addChildren([
  registerRoute,
  loginRoute,
  authCallbackRoute,
  layoutRoute.addChildren([
    indexRoute,
    subscriptionsIndexRoute,
    subscriptionsIdRoute,
    stashRoute,
    streamsRoute.addChildren([streamsIndexRoute, streamsIdRoute]),
    collectionsRoute.addChildren([collectionsIndexRoute, collectionsIdRoute]),
  ]),
])

export const buildRouter = (queryClient: QueryClient) => {
  return createRouter({
    routeTree,
    context: {
      queryClient,
    },
    defaultPreload: 'intent',
    defaultPreloadStaleTime: 0,
  })
}

declare module '@tanstack/react-router' {
  interface Register {
    router: ReturnType<typeof buildRouter>
  }

  interface HistoryState {
    registered?: boolean
    loggedOut?: boolean
  }
}

export {
  getRouteApi,
  Link,
  Outlet,
  RouterProvider,
  useParams,
  useRouteContext,
  useRouter,
  type RouteIds,
} from '@tanstack/react-router'
