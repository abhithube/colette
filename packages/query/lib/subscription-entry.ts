import type { API, SubscriptionEntryListQuery } from '@colette/core'
import { infiniteQueryOptions } from '@tanstack/react-query'

export const SUBSCRIPTION_ENTRIES_PREFIX = 'subscriptionEntries'

export const listSubscriptionEntriesOptions = (
  api: API,
  query: Omit<SubscriptionEntryListQuery, 'cursor'> = {},
) =>
  infiniteQueryOptions({
    queryKey: [SUBSCRIPTION_ENTRIES_PREFIX, query],
    queryFn: ({ pageParam }) =>
      api.subscriptionEntries.listSubscriptionEntries({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })
