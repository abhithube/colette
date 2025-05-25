import { Configuration } from 'openid-client'
import { PropsWithChildren, createContext, useContext } from 'react'

export type OidcConfig = {
  clientConfig: Configuration
  redirectUri: string
}

const OIDCConfigContext = createContext<OidcConfig | undefined>(undefined)

export const OIDCConfigProvider = (
  props: PropsWithChildren<{ oidcConfig: OidcConfig }>,
) => {
  return (
    <OIDCConfigContext.Provider value={props.oidcConfig}>
      {props.children}
    </OIDCConfigContext.Provider>
  )
}

export function useOIDCConfig(): OidcConfig {
  const oidcConfig = useContext(OIDCConfigContext)
  if (!oidcConfig) {
    throw new Error('useOIDCConfig must be used within an OIDCConfigProvider')
  }

  return oidcConfig
}
