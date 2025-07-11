import { listSubscriptionEntries } from '@colette/core/http'
import { ListSubscriptionEntriesQueryParams } from '@colette/core/types'
import { infiniteQueryOptions } from '@tanstack/react-query'

export const SUBSCRIPTION_ENTRIES_PREFIX = 'subscriptionEntries'

export const listSubscriptionEntriesOptions = (
  query: Omit<ListSubscriptionEntriesQueryParams, 'cursor'> = {},
) =>
  infiniteQueryOptions({
    queryKey: [SUBSCRIPTION_ENTRIES_PREFIX, query],
    queryFn: ({ pageParam }) =>
      listSubscriptionEntries({
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })
