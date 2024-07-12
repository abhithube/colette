import { client } from '@/lib/client'
import type { Profile } from '@/lib/types'
import type { QueryClient } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Outlet, createRootRouteWithContext } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'

export const Route = createRootRouteWithContext<{
	queryClient: QueryClient
	profile?: Profile
}>()({
	beforeLoad: async ({ context }) => {
		const data = await context.queryClient.fetchQuery({
			queryKey: ['/profiles', '@me'],
			queryFn: async ({ signal }) => {
				const res = await client.GET('/api/v1/profiles/@me', {
					signal,
				})

				return res.data
			},
		})

		context.profile = data
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
