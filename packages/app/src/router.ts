import type { API } from '@colette/core'
import type { QueryClient } from '@tanstack/react-query'
import { createRouter as createTanstackRouter } from '@tanstack/react-router'
import { routeTree } from './routeTree.gen'

export function createRouter(queryClient: QueryClient, api: API) {
  return createTanstackRouter({
    routeTree,
    context: {
      queryClient,
      api,
    },
    defaultPreload: 'intent',
  })
}

declare module '@tanstack/react-router' {
  interface Register {
    router: ReturnType<typeof createRouter>
  }
}
