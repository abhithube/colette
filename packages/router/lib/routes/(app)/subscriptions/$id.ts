import { subscriptionsRoute } from '../subscriptions'
import {
  getSubscriptionOptions,
  listSubscriptionEntriesOptions,
} from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const subscriptionsIdRoute = createRoute({
  getParentRoute: () => subscriptionsRoute,
  path: '$subscriptionId',
  loader: async ({ context, params }) => {
    await Promise.all([
      context.queryClient.ensureQueryData(
        getSubscriptionOptions(context.api, params.subscriptionId),
      ),
      context.queryClient.ensureInfiniteQueryData(
        listSubscriptionEntriesOptions(context.api, {
          subscriptionId: params.subscriptionId,
        }),
      ),
    ])
  },
})
