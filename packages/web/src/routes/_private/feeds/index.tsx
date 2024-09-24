import { ensureInfiniteQueryData, listFeedEntriesOptions } from '@colette/query'
import { HStack, Heading } from '@colette/ui'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedEntryGrid } from './-components/feed-entry-grid'

export const Route = createFileRoute('/_private/feeds/')({
  loader: async ({ context }) => {
    const options = listFeedEntriesOptions(
      { hasRead: false },
      context.profile.id,
      context.api,
    )

    await ensureInfiniteQueryData(context.queryClient, options as any)

    return {
      options,
    }
  },
  component: Component,
})

function Component() {
  const { options } = Route.useLoaderData()

  const {
    data: feedEntries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(options)

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!feedEntries) return

  return (
    <>
      <HStack pos="sticky" zIndex="sticky" top={0} bg="bg.default" p={8}>
        <Heading as="h1" fontSize="3xl" fontWeight="medium">
          All Feeds
        </Heading>
      </HStack>
      <main>
        <FeedEntryGrid
          feedEntries={feedEntries.pages.flatMap((page) => page.data)}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
