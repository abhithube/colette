import { layoutRoute } from './layout'
import { listBookmarksOptions } from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const stashRoute = createRoute({
  getParentRoute: () => layoutRoute,
  path: 'stash',
  loader: async ({ context }) => {
    await context.queryClient.ensureInfiniteQueryData(listBookmarksOptions())
  },
})
