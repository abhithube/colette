import { User } from '@colette/core/types'
import { PropsWithChildren, createContext, useContext } from 'react'

const UserContext = createContext<User | undefined>(undefined)

export const UserProvider = (props: PropsWithChildren<{ user: User }>) => {
  return (
    <UserContext.Provider value={props.user}>
      {props.children}
    </UserContext.Provider>
  )
}

export function useUser(): User {
  const user = useContext(UserContext)
  if (!user) {
    throw new Error('useUser must be used within an UserProvider')
  }

  return user
}
