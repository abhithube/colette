import { layoutRoute } from './layout'
import { listSubscriptionEntriesOptions } from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const indexRoute = createRoute({
  getParentRoute: () => layoutRoute,
  path: '/',
  loader: async ({ context }) => {
    await context.queryClient.ensureInfiniteQueryData(
      listSubscriptionEntriesOptions(),
    )
  },
})
