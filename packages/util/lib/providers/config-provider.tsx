import type { Config } from '@colette/core/types'
import { type PropsWithChildren, createContext, useContext } from 'react'

const ConfigContext = createContext<Config | undefined>(undefined)

export const ConfigProvider = (
  props: PropsWithChildren<{ config: Config }>,
) => {
  return (
    <ConfigContext.Provider value={props.config}>
      {props.children}
    </ConfigContext.Provider>
  )
}

export function useConfig(): Config {
  const config = useContext(ConfigContext)
  if (config === undefined) {
    throw new Error('useConfig must be used within an ConfigProvider')
  }

  return config
}
