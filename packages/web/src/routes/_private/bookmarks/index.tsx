import { Header, HeaderTitle } from '@/components/header'
import { ensureInfiniteQueryData, listBookmarksOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { BookmarkGrid } from './-components/bookmark-grid'

export const Route = createFileRoute('/_private/bookmarks/')({
  loader: async ({ context }) => {
    const options = listBookmarksOptions({}, context.profile.id, context.api)

    await ensureInfiniteQueryData(context.queryClient, options as any)

    return {
      options,
    }
  },
  component: Component,
})

function Component() {
  const { options } = Route.useLoaderData()

  const { data, hasNextPage, fetchNextPage } = useInfiniteQuery(options)

  const [bookmarks, setBookmarks] = useState(
    data?.pages.flatMap((page) => page.data) ?? [],
  )

  useEffect(() => {
    setBookmarks(data?.pages.flatMap((page) => page.data) ?? [])
  }, [data])

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  return (
    <>
      <Header>
        <HeaderTitle>All Bookmarks</HeaderTitle>
      </Header>
      <main>
        <BookmarkGrid
          bookmarks={bookmarks}
          setBookmarks={setBookmarks}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
