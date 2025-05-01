import { API, User } from '@colette/core'
import { getActiveUserOptions } from '@colette/query'
import type { QueryClient } from '@tanstack/react-query'
import { Link, createRootRouteWithContext } from '@tanstack/react-router'

export const rootRoute = createRootRouteWithContext<{
  api: API
  queryClient: QueryClient
  user?: User
}>()({
  beforeLoad: async ({ context }) => {
    try {
      const user = await context.queryClient.ensureQueryData(
        getActiveUserOptions(context.api),
      )

      return {
        user,
      }
    } catch (error) {
      console.error(error)
    }
  },
  notFoundComponent: () => {
    return (
      <div>
        <p>404 Not Found</p>
        <Link to="/">Home</Link>
      </div>
    )
  },
})
