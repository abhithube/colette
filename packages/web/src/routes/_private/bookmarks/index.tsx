import { HStack, Heading } from '@colette/components'
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
      <HStack pos="sticky" zIndex="sticky" top={0} bg="bg.default" p={8}>
        <Heading as="h1" fontSize="3xl" fontWeight="medium">
          All Bookmarks
        </Heading>
      </HStack>
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
