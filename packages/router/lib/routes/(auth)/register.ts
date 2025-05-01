import { rootRoute } from '../__root'
import { createRoute, redirect } from '@tanstack/react-router'

export const registerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'register',
  beforeLoad: ({ context }) => {
    if (context.user) {
      throw redirect({
        to: '/',
        replace: true,
      })
    }
  },
})
