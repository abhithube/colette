import { AppSidebar } from './sidebar'
import { getActiveUserOptions } from '@colette/query'
import { Sidebar } from '@colette/ui'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { PropsWithChildren } from 'react'
import { Redirect, useLocation } from 'wouter'

export const AuthLayout = (props: PropsWithChildren) => {
  const api = useAPI()
  const [location] = useLocation()

  const query = useQuery({
    ...getActiveUserOptions(api),
    retry: false,
  })

  if (query.isLoading) return

  if (!query.data) {
    return (
      <Redirect
        to={`/login?redirect=${encodeURIComponent(location)}`}
        replace
      />
    )
  }

  return (
    <Sidebar.Provider>
      <>
        <AppSidebar user={query.data} />
        <div className="w-full">
          <Sidebar.Trigger />
          {props.children}
        </div>
      </>
    </Sidebar.Provider>
  )
}
