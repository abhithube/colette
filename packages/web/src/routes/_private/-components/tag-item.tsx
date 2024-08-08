import {
	Tooltip,
	TooltipContent,
	TooltipProvider,
	TooltipTrigger,
} from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { Tag } from '@colette/openapi'
import { Link } from '@tanstack/react-router'

type Props = {
	tag: Tag
	type: 'bookmark' | 'feed'
}

export function TagItem({ tag, type }: Props) {
	return (
		<Link
			key={tag.id}
			className="group flex w-full items-center justify-between space-x-4 rounded-md px-4 py-2 font-medium text-sm"
			activeProps={{
				className: 'bg-muted active',
			}}
			inactiveProps={{
				className: 'hover:bg-muted/50',
			}}
			to={type === 'bookmark' ? '/bookmarks/tags/$id' : '/feeds/tags/$id'}
			params={{
				id: tag.id,
			}}
			search
		>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger asChild>
						<>
							<span className="grow truncate">{tag.title}</span>
							<span className={cn('text-muted-foreground tabular-nums')}>
								{type === 'bookmark' ? tag.bookmarkCount : tag.feedCount}
							</span>
						</>
					</TooltipTrigger>
					<TooltipContent>{tag.title}</TooltipContent>
				</Tooltip>
			</TooltipProvider>
		</Link>
	)
}
