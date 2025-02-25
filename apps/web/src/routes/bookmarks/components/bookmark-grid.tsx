import { BookmarkCard } from './bookmark-card'
import type { Bookmark } from '@colette/core'
import { useIntersectionObserver } from '@colette/util'
import type { FC } from 'react'

export const BookmarkGrid: FC<{
  bookmarks: Bookmark[]
  hasMore: boolean
  fetchMore: () => void
  created?: Bookmark
}> = (props) => {
  const target = useIntersectionObserver({
    options: {
      rootMargin: '200px',
    },
    onChange: (isIntersecting) =>
      isIntersecting && props.hasMore && props.fetchMore,
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
      {filtered.map((bookmark) => (
        <BookmarkCard key={bookmark.id} bookmark={bookmark} />
      ))}
      <div ref={target} />
    </div>
  )
}
