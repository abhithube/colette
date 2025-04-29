import { streamsRoute } from '../streams'
import {
  getStreamOptions,
  listSubscriptionEntriesOptions,
} from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const streamsIdRoute = createRoute({
  getParentRoute: () => streamsRoute,
  path: '$streamId',
  loader: async ({ context, params }) => {
    await Promise.all([
      context.queryClient.ensureQueryData(
        getStreamOptions(context.api, params.streamId),
      ),
      context.queryClient.ensureInfiniteQueryData(
        listSubscriptionEntriesOptions(context.api, {
          streamId: params.streamId,
        }),
      ),
    ])
  },
})
