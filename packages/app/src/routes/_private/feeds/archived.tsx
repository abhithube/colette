import { listFeedEntriesOptions } from '@colette/query'
import { HStack, Heading } from '@colette/ui'
import { useInfiniteQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedEntryGrid } from './-components/feed-entry-grid'

export const Route = createFileRoute('/_private/feeds/archived')({
  loader: ({ context }) => {
    const options = listFeedEntriesOptions(
      {
        hasRead: true,
      },
      context.api,
    )

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

  if (!data) return

  const feedEntries = data.pages.flatMap((page) => page.data)

  return (
    <>
      <HStack pos="sticky" zIndex="sticky" top={0} bg="bg.default" p={8}>
        <Heading as="h1" fontSize="3xl" fontWeight="medium">
          Archived
        </Heading>
      </HStack>
      <main>
        <FeedEntryGrid
          feedEntries={feedEntries}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
