import type { API } from '@colette/core'
import { getActiveOptions } from '@colette/query'
import type { QueryClient } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Outlet, createRootRouteWithContext } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'

export const Route = createRootRouteWithContext<{
  queryClient: QueryClient
  api: API
}>()({
  beforeLoad: async ({ context }) => {
    try {
      const user = await context.queryClient.fetchQuery(
        getActiveOptions(context.api),
      )

      return {
        user,
      }
    } catch (_) {
      return {
        user: undefined,
      }
    }
  },
  component: Component,
})

function Component() {
  return (
    <>
      <Outlet />
      <TanStackRouterDevtools />
      <ReactQueryDevtools />
    </>
  )
}
