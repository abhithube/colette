import { Favicon } from '@/components/favicon'
import {
	Tooltip,
	TooltipContent,
	TooltipProvider,
	TooltipTrigger,
} from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { Feed } from '@colette/openapi'
import { Link } from '@tanstack/react-router'
type Props = {
	feed: Feed
}

export function FeedItem({ feed }: Props) {
	return (
		<Link
			key={feed.id}
			className="group flex w-full items-center justify-between space-x-4 rounded-md px-4 py-2 font-medium text-sm"
			activeProps={{
				className: 'bg-muted active',
			}}
			inactiveProps={{
				className: 'hover:bg-muted/50',
			}}
			to="/feeds/$id"
			params={{
				id: feed.id,
			}}
			search
		>
			<Favicon domain={new URL(feed.link).hostname} />
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger asChild>
						<span className="grow truncate">{feed.title}</span>
					</TooltipTrigger>
					<TooltipContent>{feed.title}</TooltipContent>
				</Tooltip>
			</TooltipProvider>
			<div className="flex w-[3ch] shrink-0 justify-center">
				<span
					className={cn(
						'text-muted-foreground tabular-nums',
						feed.unreadCount === 0 && 'hidden',
					)}
				>
					{feed.unreadCount}
				</span>
			</div>
		</Link>
	)
}
