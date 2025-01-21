import { listBookmarksOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'
import { useAPI } from '../../lib/api-context'
import { BookmarkGrid } from './components/bookmark-grid'

export const StashPage: FC = () => {
  const api = useAPI()

  const { data, hasNextPage, fetchNextPage } = useInfiniteQuery(
    listBookmarksOptions({ filterByCollection: true }, api),
  )

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  return (
    <>
      <div className="sticky top-0 z-10 flex justify-between bg-background p-8">
        <h1 className="font-medium text-3xl">All Bookmarks</h1>
      </div>
      <main>
        <BookmarkGrid
          bookmarks={data?.pages.flatMap((page) => page.data) ?? []}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
