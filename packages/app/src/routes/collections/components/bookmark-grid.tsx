import type { Bookmark } from '@colette/core'
import type { FC } from 'react'
import { useInView } from 'react-intersection-observer'
import { BookmarkCard } from './bookmark-card'

export const BookmarkGrid: FC<{
  bookmarks: Bookmark[]
  hasMore: boolean
  loadMore?: () => void
  created?: Bookmark
}> = (props) => {
  const { ref } = useInView({
    threshold: 0,
    onChange: (inView) => inView && props.loadMore && props.loadMore(),
  })

  const filtered = props.created
    ? props.bookmarks.filter((v) => v.id !== props.created?.id)
    : props.bookmarks

  return (
    <div className="grid grid-cols-1 gap-4 px-8 pb-8 md:grid-cols-2 lg:grid-cols-3">
      {props.created && (
        <div className="rounded-lg border-2">
          <BookmarkCard bookmark={props.created} />
        </div>
      )}
      {filtered.map((bookmark, i) => (
        <div
          key={bookmark.id}
          ref={props.hasMore && i === filtered.length - 1 ? ref : undefined}
        >
          <BookmarkCard bookmark={bookmark} />
        </div>
      ))}
    </div>
  )
}
