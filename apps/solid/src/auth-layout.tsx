import { getActiveOptions } from '@colette/query'
import { useNavigate } from '@solidjs/router'
import { createQuery } from '@tanstack/solid-query'
import { type ParentComponent, Show, createEffect } from 'solid-js'
import { SidebarProvider, SidebarTrigger } from './components/ui/sidebar'
import { useAPI } from './lib/api-context'
import { AppSidebar } from './sidebar'

export const AuthLayout: ParentComponent = (props) => {
  const api = useAPI()
  const navigate = useNavigate()

  const query = createQuery(() => ({
    ...getActiveOptions(api),
    retry: false,
  }))

  createEffect(() => {
    if (!query.isLoading && !query.data) {
      navigate(
        `/login?redirect=${encodeURIComponent(window.location.pathname)}`,
        {
          replace: true,
        },
      )
    }
  })

  return (
    <SidebarProvider>
      <Show when={query.isLoading ? undefined : query.data}>
        {(user) => (
          <>
            <AppSidebar user={user()} />
            <div class="w-full">
              <SidebarTrigger />
              {props.children}
            </div>
          </>
        )}
      </Show>
    </SidebarProvider>
  )
}
