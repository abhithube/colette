import { layoutRoute } from './layout'
import { listStreamsOptions } from '@colette/query'
import { createRoute } from '@tanstack/react-router'

export const streamsRoute = createRoute({
  getParentRoute: () => layoutRoute,
  path: 'streams',
  loader: async ({ context }) => {
    await context.queryClient.ensureQueryData(listStreamsOptions(context.api))
  },
})
