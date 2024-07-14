import { client } from '@/lib/client'
import type { API } from '@colette/openapi'
import type { QueryClient } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Outlet, createRootRouteWithContext } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'

export const Route = createRootRouteWithContext<{
	queryClient: QueryClient
	api: API
}>()({
	beforeLoad: async ({ context }) => {
		const profile = await context.queryClient.fetchQuery({
			queryKey: ['profiles', '@me'],
			queryFn: async ({ signal }) => {
				const res = await client.GET('/api/v1/profiles/@me', {
					signal,
				})

				return res.data
			},
		})

		return {
			profile,
		}
	},
	component: Component,
})

function Component() {
	return (
		<>
			<Outlet />
			<TanStackRouterDevtools />
			<ReactQueryDevtools />
		</>
	)
}
