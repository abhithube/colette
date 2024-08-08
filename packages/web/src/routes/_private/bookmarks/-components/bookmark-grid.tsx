import type { Bookmark } from '@colette/openapi'
import { useInView } from 'react-intersection-observer'
import { BookmarkCard } from './bookmark-card'

type Props = {
	bookmarks: Bookmark[]
	hasMore: boolean
	loadMore?: () => void
	created?: Bookmark
}

export function BookmarkGrid({
	bookmarks,
	hasMore = false,
	loadMore,
	created,
}: Props) {
	const { ref } = useInView({
		threshold: 0,
		onChange: (inView) => {
			if (inView && loadMore) loadMore()
		},
	})

	if (created) {
		bookmarks = bookmarks.filter((v) => v.id !== created.id)
	}

	return (
		<>
			<div className="grid grid-cols-3 gap-4 px-8 pb-8">
				{created && (
					<div className="rounded-lg border-2 border-secondary">
						<BookmarkCard bookmark={created} />
					</div>
				)}
				{bookmarks.map((bookmark) => (
					<BookmarkCard key={bookmark.id} bookmark={bookmark} />
				))}
			</div>
			{hasMore && <div ref={ref} />}
		</>
	)
}
