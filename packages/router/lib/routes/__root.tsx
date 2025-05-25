import { API, User } from '@colette/core'
import { getActiveUserOptions, getConfigOptions } from '@colette/query'
import type { QueryClient } from '@tanstack/react-query'
import { Link, createRootRouteWithContext } from '@tanstack/react-router'
import * as client from 'openid-client'

export const rootRoute = createRootRouteWithContext<{
  api: API
  queryClient: QueryClient
  user?: User
}>()({
  beforeLoad: async ({ context }) => {
    const config = await context.queryClient.ensureQueryData(
      getConfigOptions(context.api),
    )

    const clientConfig = await client.discovery(
      new URL(config.oidc.issuer_url),
      config.oidc.client_id,
      undefined,
      undefined,
      {
        execute: [client.allowInsecureRequests],
      },
    )

    try {
      const user = await context.queryClient.ensureQueryData(
        getActiveUserOptions(context.api),
      )

      return {
        user,
        oidcConfig: {
          clientConfig,
          redirectUri: config.oidc.redirect_url,
        },
      }
    } catch (error) {
      console.error(error)

      return {
        oidcConfig: {
          clientConfig,
          redirectUri: config.oidc.redirect_url,
        },
      }
    }
  },
  notFoundComponent: () => {
    return (
      <div>
        <p>404 Not Found</p>
        <Link to="/">Home</Link>
      </div>
    )
  },
})
