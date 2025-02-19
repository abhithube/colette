import type { API, LibraryItemListQuery } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

const LIBRARY_PREFIX = 'library'

export const listLibraryItemsOptions = (
  api: API,
  query?: LibraryItemListQuery,
) =>
  queryOptions({
    queryKey: [LIBRARY_PREFIX, query],
    queryFn: () => api.library.list(query ?? {}),
  })
