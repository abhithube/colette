import { layoutRoute } from './layout'
import { listSubscriptionsOptions } from '@colette/query'
import { createRoute, Outlet } from '@tanstack/react-router'

export const subscriptionsRoute = createRoute({
  getParentRoute: () => layoutRoute,
  path: 'subscriptions',
  loader: async ({ context }) => {
    await context.queryClient.ensureQueryData(
      listSubscriptionsOptions(context.api),
    )
  },
  component: () => <Outlet />,
})
