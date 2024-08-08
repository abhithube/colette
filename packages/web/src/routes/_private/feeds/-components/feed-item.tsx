import { Favicon } from '@/components/favicon'
import { SidebarLink } from '@/components/sidebar'
import {
	Tooltip,
	TooltipContent,
	TooltipProvider,
	TooltipTrigger,
} from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { Feed } from '@colette/openapi'

type Props = {
	feed: Feed
}

export function FeedItem({ feed }: Props) {
	return (
		<SidebarLink
			to="/feeds/$id"
			params={{
				id: feed.id,
			}}
		>
			<Favicon domain={new URL(feed.link).hostname} />
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger className="grow truncate text-left">
						{feed.title}
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
		</SidebarLink>
	)
}
