import { rootRoute } from '../__root'
import { createRoute, redirect } from '@tanstack/react-router'
import { zodValidator } from '@tanstack/zod-adapter'
import * as client from 'openid-client'
import { z } from 'zod'

export const authCallbackRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'auth-callback',
  validateSearch: zodValidator(
    z.object({
      code: z.string(),
      state: z.string(),
      iss: z.string().url().optional(),
      session_state: z.string().optional(),
    }),
  ),
  beforeLoad: async ({ context }) => {
    if (!context.oidcConfig) {
      throw redirect({
        to: '/',
        replace: true,
      })
    }

    if (!context.user) {
      const codeVerifier =
        sessionStorage.getItem('colette-code-verifier') ?? undefined

      const res = await client.authorizationCodeGrant(
        context.oidcConfig.clientConfig,
        new URL(window.location.href),
        {
          pkceCodeVerifier: codeVerifier,
        },
      )

      sessionStorage.removeItem('colette-code-verifier')

      localStorage.setItem('colette-access-token', res.access_token)
      if (res.refresh_token) {
        localStorage.setItem('colette-refresh-token', res.refresh_token)
      }
    }

    const from = sessionStorage.getItem('colette-from')
    if (from) {
      sessionStorage.removeItem('colette-from')
    }

    throw redirect({
      to: from ?? '/',
      replace: true,
    })
  },
})
