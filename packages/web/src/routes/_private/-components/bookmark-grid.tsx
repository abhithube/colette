import type { Bookmark } from '@colette/openapi'
import { useInView } from 'react-intersection-observer'
import { BookmarkCard } from './bookmark-card'

type Props = {
	bookmarks: Bookmark[]
	hasMore: boolean
	loadMore?: () => void
}

export function BookmarkGrid({ bookmarks, hasMore = false, loadMore }: Props) {
	const { ref } = useInView({
		threshold: 0,
		onChange: (inView) => {
			if (inView && loadMore) loadMore()
		},
	})

	return (
		<>
			<div className="grid grid-cols-3 gap-4 px-8">
				{bookmarks.map((bookmark) => (
					<BookmarkCard key={bookmark.id} bookmark={bookmark} />
				))}
			</div>
			{hasMore && <div ref={ref} />}
		</>
	)
}
