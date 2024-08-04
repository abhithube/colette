import {
	Tooltip,
	TooltipContent,
	TooltipProvider,
	TooltipTrigger,
} from '@/components/ui/tooltip'
import type { Tag } from '@colette/openapi'
import { Link } from '@tanstack/react-router'

type Props = {
	tag: Tag
}

export function TagItem({ tag }: Props) {
	return (
		<Link
			key={tag.id}
			className="group flex w-full items-center justify-between space-x-4 rounded-md px-4 py-2 font-medium text-primary text-sm"
			activeProps={{
				className: 'bg-muted active text-secondary',
			}}
			inactiveProps={{
				className: 'hover:bg-muted/50',
			}}
			to="/bookmarks/tags/$id"
			params={{
				id: tag.id,
			}}
			search
		>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger asChild>
						<span className="grow truncate">{tag.title}</span>
					</TooltipTrigger>
					<TooltipContent>{tag.title}</TooltipContent>
				</Tooltip>
			</TooltipProvider>
		</Link>
	)
}
