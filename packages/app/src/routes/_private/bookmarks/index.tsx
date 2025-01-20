import { listBookmarksOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { BookmarkGrid } from './-components/bookmark-grid'

export const Route = createFileRoute('/_private/bookmarks/')({
  loader: ({ context }) => {
    const options = listBookmarksOptions({}, context.api)

    return {
      options,
    }
  },
  component: Component,
})

function Component() {
  const { options } = Route.useLoaderData()

  const { data, hasNextPage, fetchNextPage } = useInfiniteQuery(options)

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
