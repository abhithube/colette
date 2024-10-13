import {
  ensureInfiniteQueryData,
  getSmartFeedOptions,
  listFeedEntriesOptions,
} from '@colette/query'
import { HStack, Heading } from '@colette/ui'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useEffect } from 'react'
import { FeedEntryGrid } from '../-components/feed-entry-grid'

export const Route = createFileRoute('/_private/feeds/smartFeeds/$id')({
  loader: async ({ context, params }) => {
    const smartFeedOptions = getSmartFeedOptions(params.id, context.api)

    const feedEntryOptions = listFeedEntriesOptions(
      {
        smartFeedId: params.id,
        hasRead: false,
      },
      context.profile.id,
      context.api,
    )

    await Promise.all([
      context.queryClient.ensureQueryData(smartFeedOptions),
      ensureInfiniteQueryData(context.queryClient, feedEntryOptions as any),
    ])

    return {
      smartFeedOptions,
      feedEntryOptions,
    }
  },
  component: Component,
})

function Component() {
  const { id } = Route.useParams()
  const { smartFeedOptions, feedEntryOptions } = Route.useLoaderData()

  const { data: smartFeed } = useQuery(smartFeedOptions)
  const {
    data: feedEntries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(feedEntryOptions)

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [id])

  if (!smartFeed || !feedEntries) return

  return (
    <>
      <HStack
        pos="sticky"
        zIndex="sticky"
        top={0}
        justify="space-between"
        bg="bg.default"
        p={8}
      >
        <Heading as="h1" fontSize="3xl" fontWeight="medium" lineClamp={1}>
          {smartFeed.title}
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
