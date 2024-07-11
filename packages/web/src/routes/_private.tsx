import { Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { OuterSidebar } from './-components/outer-sidebar'

export const Route = createFileRoute('/_private')({
	beforeLoad: async ({ context }) => {
		if (!context.profile) {
			throw redirect({
				to: '/login',
			})
		}
	},
	component: Component,
})

function Component() {
	const { profile } = Route.useRouteContext()

	if (!profile) return

	return (
		<div className="flex h-screen">
			<OuterSidebar profile={profile} />
			<div className="w-full overflow-y-auto">
				<Outlet />
			</div>
		</div>
	)
}
