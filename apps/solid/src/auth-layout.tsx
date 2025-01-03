import { getActiveOptions } from '@colette/solid-query'
import { useNavigate } from '@solidjs/router'
import { createQuery } from '@tanstack/solid-query'
import { type ParentComponent, Show, createEffect } from 'solid-js'
import { SidebarProvider, SidebarTrigger } from './components/ui/sidebar'
import { useAPI } from './lib/api-context'
import { AppSidebar } from './sidebar'

export const AuthLayout: ParentComponent = (props) => {
  const navigate = useNavigate()

  const query = createQuery(() => ({
    ...getActiveOptions(useAPI()),
    retry: false,
  }))

  createEffect(() => {
    if (!query.isLoading && !query.data) {
      const currentPath = window.location.pathname
      navigate(`/login?redirect=${encodeURIComponent(currentPath)}`, {
        replace: true,
      })
    }
  })

  return (
    <Show when={!query.isLoading && !!query.data}>
      <SidebarProvider>
        <AppSidebar user={query.data!} />
        <div class="w-full">
          <SidebarTrigger />
          {props.children}
        </div>
      </SidebarProvider>
    </Show>
  )
}
