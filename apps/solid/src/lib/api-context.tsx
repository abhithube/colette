import type { API } from '@colette/core'
import { type ParentComponent, createContext, useContext } from 'solid-js'

const APIContext = createContext<API>()

export const APIProvider: ParentComponent<{ api: API }> = (props) => {
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
