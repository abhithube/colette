import type { API } from '@colette/core'
import { PropsWithChildren, createContext, useContext } from 'react'

const APIContext = createContext<API | undefined>(undefined)

export const APIProvider = (props: PropsWithChildren<{ api: API }>) => {
  return (
    <APIContext.Provider value={props.api}>
      {props.children}
    </APIContext.Provider>
  )
}

export function useAPI(): API {
  const api = useContext(APIContext)
  if (!api) {
    throw new Error('useAPI must be used within an APIProvider')
  }

  return api
}
