import { streamsRoute } from '../streams'
import { createRoute } from '@tanstack/react-router'

export const streamsIndexRoute = createRoute({
  getParentRoute: () => streamsRoute,
  path: '/',
})
