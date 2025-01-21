import { useAPI } from '../../lib/api-context'
import { BookmarkGrid } from './components/bookmark-grid'
import { listBookmarksOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'

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
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">All Bookmarks</h1>
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
