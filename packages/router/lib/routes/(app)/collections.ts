import { layoutRoute } from './layout'
import { listCollectionsOptions } from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const collectionsRoute = createRoute({
  getParentRoute: () => layoutRoute,
  path: 'collections',
  loader: async ({ context }) => {
    await context.queryClient.ensureQueryData(listCollectionsOptions())
  },
})
