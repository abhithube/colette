import { client as fetchClient } from '@colette/core/client'
import { getActiveUserOptions, getConfigOptions } from '@colette/query'
import { QueryClient } from '@tanstack/react-query'
import { Link, createRootRouteWithContext } from '@tanstack/react-router'
import * as oidcClient from 'openid-client'

export const rootRoute = createRootRouteWithContext<{
  queryClient: QueryClient
}>()({
  beforeLoad: async ({ context }) => {
    const config = await context.queryClient.ensureQueryData(getConfigOptions())

    let oidcConfig:
      | { clientConfig: oidcClient.Configuration; redirectUri: string }
      | undefined

    if (config.oidc) {
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

      oidcConfig = {
        clientConfig,
        redirectUri: config.oidc.redirectUri,
      }
    }

    try {
      const user = await context.queryClient.ensureQueryData(
        getActiveUserOptions(),
      )

      return {
        user,
        oidcConfig,
      }
    } catch (error) {
      console.error(error)

      return {
        oidcConfig,
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
