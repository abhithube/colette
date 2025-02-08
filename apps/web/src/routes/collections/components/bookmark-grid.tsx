import { BookmarkCard } from './bookmark-card'
import type { Bookmark, BookmarkListQuery } from '@colette/core'
import { listBookmarksOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useIntersectionObserver } from '@colette/util'
import { useInfiniteQuery } from '@tanstack/react-query'
import type { FC } from 'react'

export const BookmarkGrid: FC<{
  query: BookmarkListQuery
  created?: Bookmark
}> = (props) => {
  const api = useAPI()

  const { data, isLoading, hasNextPage, fetchNextPage } = useInfiniteQuery(
    listBookmarksOptions(props.query, api),
  )

  const target = useIntersectionObserver({
    options: {
      rootMargin: '200px',
    },
    onChange: (isIntersecting) =>
      isIntersecting && hasNextPage && fetchNextPage,
  })

  if (isLoading || !data) return

  const bookmarks = data.pages.flatMap((page) => page.data)

  const filtered = props.created
    ? bookmarks.filter((v) => v.id !== props.created?.id)
    : bookmarks

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
