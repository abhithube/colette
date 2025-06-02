import { client as fetchClient, User } from '@colette/core'
import { getActiveUserOptions, getConfigOptions } from '@colette/query'
import type { QueryClient } from '@tanstack/react-query'
import { Link, createRootRouteWithContext } from '@tanstack/react-router'
import * as oidcClient from 'openid-client'

export const rootRoute = createRootRouteWithContext<{
  queryClient: QueryClient
  user?: User
}>()({
  beforeLoad: async ({ context }) => {
    const config = await context.queryClient.ensureQueryData(getConfigOptions())

    const clientConfig = await oidcClient.discovery(
      new URL(config.oidc.issuer),
      config.oidc.clientId,
      undefined,
      undefined,
      {
        execute: [oidcClient.allowInsecureRequests],
      },
    )

    fetchClient.setConfig({
      ...fetchClient.getConfig(),
      oidcConfig: clientConfig,
    })

    try {
      const user = await context.queryClient.ensureQueryData(
        getActiveUserOptions(),
      )

      return {
        user,
        oidcConfig: {
          clientConfig,
          redirectUri: config.oidc.redirectUri,
        },
      }
    } catch (error) {
      console.error(error)

      return {
        oidcConfig: {
          clientConfig,
          redirectUri: config.oidc.redirectUri,
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
