import { rootRoute } from '../__root'
import { createRoute, redirect } from '@tanstack/react-router'

export const layoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: 'layout',
  beforeLoad: async ({ context, location }) => {
    if (!context.user) {
      throw redirect({
        to: '/login',
        search: {
          from: location.href,
        },
        replace: true,
      })
    }

    return {
      user: context.user,
    }
  },
})
