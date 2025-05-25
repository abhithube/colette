import { rootRoute } from '../__root'
import { createRoute, redirect } from '@tanstack/react-router'
import { zodValidator } from '@tanstack/zod-adapter'
import { z } from 'zod'

export const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'login',
  validateSearch: zodValidator(
    z.object({
      loggedOut: z.boolean().optional(),
    }),
  ),
  beforeLoad: ({ context }) => {
    if (context.user) {
      throw redirect({
        to: '/',
        replace: true,
      })
    }
  },
})
