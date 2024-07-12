import { Favicon } from '@/components/favicon'
import type { Feed } from '@/lib/types'
import { Link } from '@tanstack/react-router'

type Props = {
	feed: Feed
}

export function FeedItem({ feed }: Props) {
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
		>
			<Favicon domain={new URL(feed.link).hostname} />
			<span className="grow truncate">{feed.title}</span>
		</Link>
	)
}
