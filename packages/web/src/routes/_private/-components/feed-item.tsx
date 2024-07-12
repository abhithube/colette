import { Favicon } from '@/components/favicon'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuShortcut,
	DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import type { Feed } from '@/lib/types'
import { cn } from '@/lib/utils'
import { Link } from '@tanstack/react-router'
import { MoreHorizontal } from 'lucide-react'
import { useState } from 'react'
import { UnsubscribeAlert } from './unsubscribe-alert'

type Props = {
	feed: Feed
}

export function FeedItem({ feed }: Props) {
	const [isHovering, setHovering] = useState(false)
	const [isDropdownOpen, setDropdownOpen] = useState(false)
	const [isUnsubscribeAlertOpen, setUnsubscribeAlertOpen] = useState(false)

	return (
		<Link
			key={feed.id}
			className="group flex w-full items-center justify-between space-x-4 rounded-md px-4 py-2 font-medium text-primary text-sm"
			activeProps={{
				className: 'bg-muted active text-secondary',
			}}
			inactiveProps={{
				className: 'hover:bg-muted/50',
			}}
			to="/feeds/$id"
			params={{
				id: feed.id,
			}}
			search
			onMouseEnter={() => setHovering(true)}
			onMouseLeave={() => setHovering(false)}
		>
			<Favicon domain={new URL(feed.link).hostname} />
			<span className="grow truncate">{feed.title}</span>
			<div className="flex w-[3ch] shrink-0 justify-center">
				<DropdownMenu open={isDropdownOpen} onOpenChange={setDropdownOpen}>
					<DropdownMenuTrigger>
						{isHovering || isDropdownOpen ? (
							<MoreHorizontal className="h-5 text-muted-foreground hover:text-primary" />
						) : (
							<span
								className={cn(
									'text-muted-foreground tabular-nums group-[.active]:text-secondary',
									feed.unreadCount === 0 && 'hidden',
								)}
							>
								{feed.unreadCount}
							</span>
						)}
					</DropdownMenuTrigger>
					<DropdownMenuContent className="w-56">
						<DropdownMenuItem
							onClick={(e) => {
								e.stopPropagation()

								window.open(feed.link)
							}}
						>
							Open in new tab
							<DropdownMenuShortcut>⇧⌘O</DropdownMenuShortcut>
						</DropdownMenuItem>
						<DropdownMenuItem>
							Mark all as read
							<DropdownMenuShortcut>⇧⌘R</DropdownMenuShortcut>
						</DropdownMenuItem>
						<DropdownMenuSeparator />
						<DropdownMenuItem
							onClick={(e) => {
								e.stopPropagation()

								setUnsubscribeAlertOpen(true)
							}}
						>
							Unsubscribe
							<DropdownMenuShortcut>⇧⌘O</DropdownMenuShortcut>
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
				<UnsubscribeAlert
					feed={feed}
					isOpen={isUnsubscribeAlertOpen}
					setOpen={setUnsubscribeAlertOpen}
				/>
			</div>
		</Link>
	)
}
