import { Button } from '@/components/ui/button'
import {
	ResizableHandle,
	ResizablePanel,
	ResizablePanelGroup,
} from '@/components/ui/resizable'
import { Separator } from '@/components/ui/separator'
import { client } from '@/lib/client'
import { type QueryOptions, useQuery } from '@tanstack/react-query'
import { Outlet, createFileRoute } from '@tanstack/react-router'
import { History, Home, Plus } from 'lucide-react'
import { SidebarLink } from '../-components/sidebar-link'
import { FeedItem } from './-components/feed-item'

export const options = (profileId: string) => {
	return {
		queryKey: ['/profiles', profileId, '/feeds'],
		queryFn: async ({ signal }) => {
			const res = await client.GET('/api/v1/feeds', {
				signal,
			})

			return res.data
		},
	} satisfies QueryOptions
}

export const Route = createFileRoute('/_private/feeds')({
	loader: async ({ context }) => {
		await context.queryClient.ensureQueryData(options(context.profile.id))
	},
	component: Component,
})

function Component() {
	const { profile } = Route.useRouteContext()

	const { data: feeds } = useQuery(options(profile.id))

	if (!feeds) return

	return (
		<div className="flex h-full w-full">
			<ResizablePanelGroup direction="horizontal">
				<ResizablePanel minSize={15} defaultSize={20} maxSize={30} collapsible>
					<div className="space-y-1 p-4">
						<SidebarLink to="/feeds" activeOptions={{ exact: true }}>
							<Home className="h-4 w-4 shrink-0" />
							<span className="grow truncate">All Feeds</span>
						</SidebarLink>
						<SidebarLink to="/feeds/archived">
							<History className="h-4 w-4 shrink-0" />
							<span className="grow truncate">Archived</span>
						</SidebarLink>
					</div>
					<Separator />
					<div className="h-full pt-4">
						<div className="flex h-8 items-center justify-between px-4">
							<span className="grow font-semibold text-muted-foreground text-xs">
								Feeds
							</span>
							<Button className="h-8 w-8 justify-center" variant="ghost">
								<Plus className="h-4 w-4 shrink-0" />
							</Button>
						</div>
						<div className="mt-1 h-full space-y-1 overflow-y-auto px-4">
							{feeds.data.length > 0 ? (
								<>
									{feeds.data.map((feed) => (
										<FeedItem key={feed.id} feed={feed} />
									))}
								</>
							) : (
								<div className="font-light text-sm">
									You have not subscribed to any feeds yet. Click + to add one.
								</div>
							)}
						</div>
					</div>
				</ResizablePanel>
				<ResizableHandle />
				<ResizablePanel>
					<div className="h-screen overflow-y-auto">
						<Outlet />
					</div>
				</ResizablePanel>
			</ResizablePanelGroup>
		</div>
	)
}
