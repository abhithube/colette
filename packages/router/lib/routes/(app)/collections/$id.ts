import { collectionsRoute } from '../collections'
import { getCollectionOptions, listBookmarksOptions } from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const collectionsIdRoute = createRoute({
  getParentRoute: () => collectionsRoute,
  path: '$collectionId',
  loader: async ({ context, params }) => {
    await Promise.all([
      context.queryClient.ensureQueryData(
        getCollectionOptions(params.collectionId),
      ),
      context.queryClient.ensureInfiniteQueryData(
        listBookmarksOptions({
          collectionId: params.collectionId,
        }),
      ),
    ])
  },
})
