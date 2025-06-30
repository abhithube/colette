import { layoutRoute } from '../layout'
import { listSubscriptionsOptions } from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const subscriptionsIndexRoute = createRoute({
  getParentRoute: () => layoutRoute,
  path: 'subscriptions',
  loader: async ({ context }) => {
    await context.queryClient.ensureQueryData(listSubscriptionsOptions())
  },
})
