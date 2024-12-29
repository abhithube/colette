import { getActiveOptions } from '@colette/solid-query'
import { useNavigate } from '@solidjs/router'
import { createQuery } from '@tanstack/solid-query'
import { type ParentComponent, Show, createEffect } from 'solid-js'
import { useAPI } from './lib/api-context'

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

  return <Show when={!query.isLoading && !!query.data}>{props.children}</Show>
}
