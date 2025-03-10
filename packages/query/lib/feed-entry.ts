import type { API, FeedEntryListQuery } from '@colette/core'
import { infiniteQueryOptions } from '@tanstack/react-query'

const FEED_ENTRIES_PREFIX = 'feedEntries'

export const listFeedEntriesOptions = (
  api: API,
  query: Omit<FeedEntryListQuery, 'cursor'> = {},
) =>
  infiniteQueryOptions({
    queryKey: [FEED_ENTRIES_PREFIX, query],
    queryFn: ({ pageParam }) =>
      api.feedEntries.listFeedEntries({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })
