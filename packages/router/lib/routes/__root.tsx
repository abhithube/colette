import { getActiveUserOptions, getConfigOptions } from '@colette/query'
import { QueryClient } from '@tanstack/react-query'
import { Link, createRootRouteWithContext } from '@tanstack/react-router'

export const rootRoute = createRootRouteWithContext<{
  queryClient: QueryClient
}>()({
  beforeLoad: async ({ context }) => {
    const config = await context.queryClient.ensureQueryData(getConfigOptions())

    try {
      const user = await context.queryClient.ensureQueryData(
        getActiveUserOptions(),
      )

      return {
        user,
        config,
      }
    } catch {
      return {
        config,
      }
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
