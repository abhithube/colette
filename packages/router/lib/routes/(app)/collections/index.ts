import { collectionsRoute } from '../collections'
import { createRoute } from '@tanstack/react-router'

export const collectionsIndexRoute = createRoute({
  getParentRoute: () => collectionsRoute,
  path: '/',
})
