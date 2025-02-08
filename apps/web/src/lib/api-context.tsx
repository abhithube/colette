import type { API } from '@colette/core'
import {
  type FC,
  type PropsWithChildren,
  createContext,
  useContext,
} from 'react'

const APIContext = createContext<API | undefined>(undefined)

export const APIProvider: FC<PropsWithChildren<{ api: API }>> = (props) => {
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
