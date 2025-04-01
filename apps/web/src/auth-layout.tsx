import { AppSidebar } from './sidebar'
import { getActiveUserOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { type FC, type PropsWithChildren } from 'react'
import { Redirect, useLocation } from 'wouter'
import { SidebarProvider, SidebarTrigger } from '~/components/ui/sidebar'

export const AuthLayout: FC<PropsWithChildren> = (props) => {
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
    <SidebarProvider>
      <>
        <AppSidebar user={query.data} />
        <div className="w-full">
          <SidebarTrigger />
          {props.children}
        </div>
      </>
    </SidebarProvider>
  )
}
