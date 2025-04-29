import { collectionsRoute } from '../collections'
import { getCollectionOptions, listBookmarksOptions } from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const collectionsIdRoute = createRoute({
  getParentRoute: () => collectionsRoute,
  path: '$collectionId',
  loader: async ({ context, params }) => {
    await Promise.all([
      context.queryClient.ensureQueryData(
        getCollectionOptions(context.api, params.collectionId),
      ),
      context.queryClient.ensureInfiniteQueryData(
        listBookmarksOptions(context.api, {
          collectionId: params.collectionId,
        }),
      ),
    ])
  },
})
