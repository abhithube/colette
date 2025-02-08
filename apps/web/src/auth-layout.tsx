import { useAPI } from './lib/api-context'
import { AppSidebar } from './sidebar'
import { getActiveOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { type FC, type PropsWithChildren, useEffect } from 'react'
import { useLocation } from 'wouter'
import { SidebarProvider, SidebarTrigger } from '~/components/ui/sidebar'

export const AuthLayout: FC<PropsWithChildren> = (props) => {
  const api = useAPI()
  const [, navigate] = useLocation()

  const { data, isLoading } = useQuery({
    ...getActiveOptions(api),
    retry: false,
  })

  useEffect(() => {
    if (!isLoading && !data) {
      navigate(
        `/login?redirect=${encodeURIComponent(window.location.pathname)}`,
        {
          replace: true,
        },
      )
    }
  }, [isLoading, data, navigate])

  if (isLoading || !data) return

  return (
    <SidebarProvider>
      <>
        <AppSidebar user={data} />
        <div className="w-full">
          <SidebarTrigger />
          {props.children}
        </div>
      </>
    </SidebarProvider>
  )
}
