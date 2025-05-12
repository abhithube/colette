import { BookmarkCard } from './bookmark-card'
import type { Bookmark, BookmarkDetails } from '@colette/core'
import { useIntersectionObserver } from '@colette/util'

export const BookmarkGrid = (props: {
  bookmarks: BookmarkDetails[]
  hasMore: boolean
  fetchMore: () => void
  created?: Bookmark
}) => {
  const target = useIntersectionObserver({
    options: {
      rootMargin: '200px',
    },
    onChange: (isIntersecting) =>
      isIntersecting && props.hasMore && props.fetchMore,
  })

  const filtered = props.created
    ? props.bookmarks.filter((v) => v.bookmark.id !== props.created?.id)
    : props.bookmarks

  return (
    <div className="grid grid-cols-1 gap-4 px-8 pb-8 md:grid-cols-2 lg:grid-cols-3">
      {props.created && (
        <div className="rounded-lg border-2">
          <BookmarkCard details={{ bookmark: props.created }} />
        </div>
      )}
      {filtered.map((details) => (
        <BookmarkCard key={details.bookmark.id} details={details} />
      ))}
      <div ref={target} />
    </div>
  )
}
