import { Button } from '@/components/ui/button'
import { Dialog, DialogTrigger } from '@/components/ui/dialog'
import {
	ResizableHandle,
	ResizablePanel,
	ResizablePanelGroup,
} from '@/components/ui/resizable'
import { Separator } from '@/components/ui/separator'
import { Outlet, createFileRoute } from '@tanstack/react-router'
import { History, Home, Plus } from 'lucide-react'
import { useState } from 'react'
import { SidebarLink } from '../-components/sidebar-link'

export const Route = createFileRoute('/_private/bookmarks')({
	component: Component,
})

function Component() {
	const [isOpen, setOpen] = useState(false)

	return (
		<div className="flex h-full w-full">
			<ResizablePanelGroup direction="horizontal">
				<ResizablePanel minSize={15} defaultSize={20} maxSize={30} collapsible>
					<div className="space-y-1 p-4">
						<SidebarLink to="/bookmarks" activeOptions={{ exact: true }}>
							<Home className="h-4 w-4 shrink-0" />
							<span className="grow truncate">All Bookmarks</span>
						</SidebarLink>
						<SidebarLink to="/bookmarks/stash">
							<History className="h-4 w-4 shrink-0" />
							<span className="grow truncate">Stash</span>
						</SidebarLink>
					</div>
					<Separator />
					<div className="h-full pt-4">
						<div className="flex h-8 items-center justify-between px-4">
							<span className="grow font-semibold text-muted-foreground text-xs">
								Bookmarks
							</span>
							<Dialog open={isOpen} onOpenChange={setOpen}>
								<DialogTrigger asChild>
									<Button className="h-8 w-8 justify-center" variant="ghost">
										<Plus className="h-4 w-4 shrink-0" />
									</Button>
								</DialogTrigger>
							</Dialog>
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
