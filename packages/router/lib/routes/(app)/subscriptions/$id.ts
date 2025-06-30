import { layoutRoute } from '../layout'
import {
  getSubscriptionOptions,
  listSubscriptionEntriesOptions,
} from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const subscriptionsIdRoute = createRoute({
  getParentRoute: () => layoutRoute,
  path: 'subscriptions/$subscriptionId',
  loader: async ({ context, params }) => {
    await Promise.all([
      context.queryClient.ensureQueryData(
        getSubscriptionOptions(params.subscriptionId, {
          withFeed: true,
        }),
      ),
      context.queryClient.ensureInfiniteQueryData(
        listSubscriptionEntriesOptions({
          subscriptionId: params.subscriptionId,
        }),
      ),
    ])
  },
})
