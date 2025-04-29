import { subscriptionsRoute } from '../subscriptions'
import { createRoute } from '@tanstack/react-router'

export const subscriptionsIndexRoute = createRoute({
  getParentRoute: () => subscriptionsRoute,
  path: '/',
})
