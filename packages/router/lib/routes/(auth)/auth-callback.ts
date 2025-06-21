import { rootRoute } from '../__root'
import client from '@colette/core/client'
import { exchangeCode } from '@colette/core/http'
import { createRoute, redirect } from '@tanstack/react-router'
import { zodValidator } from '@tanstack/zod-adapter'
import { z } from 'zod'

export const authCallbackRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'auth-callback',
  validateSearch: zodValidator(
    z.object({
      code: z.string(),
      state: z.string(),
    }),
  ),
  beforeLoad: async ({ context, search }) => {
    if (!context.config.oidc) {
      throw redirect({
        to: '/',
        replace: true,
      })
    }

    if (!context.user) {
      const data = await exchangeCode({
        code: search.code,
        state: search.state,
      })

      client.setConfig({
        ...client.getConfig(),
        accessToken: data.accessToken,
      })
    }

    const from = sessionStorage.getItem('colette_from')
    if (from) {
      sessionStorage.removeItem('colette_from')
    }

    throw redirect({
      to: from ?? '/',
      replace: true,
    })
  },
})
