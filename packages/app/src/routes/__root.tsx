import type { API } from '@colette/core'
import { getActiveProfileOptions } from '@colette/query'
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
      const profile = await context.queryClient.fetchQuery(
        getActiveProfileOptions(context.api),
      )

      return {
        profile,
      }
    } catch (_) {
      return {
        profile: undefined,
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
