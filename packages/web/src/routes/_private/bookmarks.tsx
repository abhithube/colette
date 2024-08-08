import { Icon } from '@/components/icon'
import { Button } from '@/components/ui/button'
import { Dialog, DialogTrigger } from '@/components/ui/dialog'
import {
	ResizableHandle,
	ResizablePanel,
	ResizablePanelGroup,
} from '@/components/ui/resizable'
import { Separator } from '@/components/ui/separator'
import { listTagsOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { Outlet, createFileRoute } from '@tanstack/react-router'
import { History, Home, Plus, PlusCircle } from 'lucide-react'
import { useState } from 'react'
import { SidebarLink } from '../../components/sidebar'
import { AddTagModal } from './-components/add-tag-modal'
import { TagItem } from './-components/tag-item'
import { AddBookmarkModal } from './bookmarks/-components/add-bookmark-modal'

export const Route = createFileRoute('/_private/bookmarks')({
	loader: async ({ context }) => {
		const options = listTagsOptions(
			{ tagType: 'bookmarks' },
			context.profile.id,
			context.api,
		)

		await context.queryClient.ensureQueryData(options)

		return {
			options,
		}
	},
	component: Component,
})

function Component() {
	const { options } = Route.useLoaderData()

	const { data: tags } = useQuery(options)

	const [isBookmarkModalOpen, setBookmarkModalOpen] = useState(false)
	const [isTagModalOpen, setTagModalOpen] = useState(false)

	if (!tags) return

	return (
		<div className="flex h-full w-full">
			<ResizablePanelGroup direction="horizontal">
				<ResizablePanel minSize={15} defaultSize={20} maxSize={30} collapsible>
					<div className="h-full space-y-4 overflow-y-auto py-4">
						<div className="flex items-center justify-between px-4">
							<h1 className="font-medium text-3xl">Bookmarks</h1>
							<Dialog
								open={isBookmarkModalOpen}
								onOpenChange={setBookmarkModalOpen}
							>
								<DialogTrigger asChild>
									<Button className="space-x-2">
										<span className="text-sm">New</span>
										<Icon value={PlusCircle} />
									</Button>
								</DialogTrigger>
								<AddBookmarkModal close={() => setBookmarkModalOpen(false)} />
							</Dialog>
						</div>
						<div className="space-y-1 px-4">
							<SidebarLink to="/bookmarks" activeOptions={{ exact: true }}>
								<Icon value={Home} />
								<span className="grow truncate">All Bookmarks</span>
							</SidebarLink>
							<SidebarLink to="/bookmarks/stash">
								<Icon value={History} />
								<span className="grow truncate">Stash</span>
							</SidebarLink>
						</div>
						<Separator />
						<div>
							<div className="flex h-8 items-center justify-between px-4">
								<span className="grow font-semibold text-muted-foreground text-xs">
									Tags
								</span>
								<Dialog open={isTagModalOpen} onOpenChange={setTagModalOpen}>
									<DialogTrigger asChild>
										<Button className="h-8 w-8 justify-center" variant="ghost">
											<Icon value={Plus} />
										</Button>
									</DialogTrigger>
									<AddTagModal close={() => setTagModalOpen(false)} />
								</Dialog>
							</div>
							<div className="mt-1 space-y-1 px-4">
								{tags.data.length > 0 ? (
									<>
										{tags.data.map((tag) => (
											<TagItem key={tag.id} tag={tag} type="bookmark" />
										))}
									</>
								) : (
									<div className="font-light text-sm">
										You have not created any tags yet. Click + to add one.
									</div>
								)}
							</div>
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
