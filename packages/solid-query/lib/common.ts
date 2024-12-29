import type { QueryClient } from '@tanstack/query-core'
import type { FetchInfiniteQueryOptions } from '@tanstack/solid-query'

export const ensureInfiniteQueryData = async (
  queryClient: QueryClient,
  options: FetchInfiniteQueryOptions,
) => {
  const data = queryClient.getQueryData(options.queryKey)
  if (!data) {
    await queryClient.fetchInfiniteQuery(options)
  }
}
