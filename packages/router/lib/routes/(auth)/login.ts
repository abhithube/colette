import { rootRoute } from '../__root'
import { createRoute, redirect } from '@tanstack/react-router'
import { zodValidator } from '@tanstack/zod-adapter'
import { z } from 'zod'

export const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'login',
  validateSearch: zodValidator(
    z.object({
      from: z.string().optional(),
    }),
  ),
  beforeLoad: ({ context, search }) => {
    if (context.user) {
      if (search.from) {
        throw redirect({
          to: search.from,
          replace: true,
        })
      }

      throw redirect({
        to: '/',
        replace: true,
      })
    }
  },
})
