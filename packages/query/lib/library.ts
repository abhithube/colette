import type { API, LibraryItemListQuery } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

const FOLDERS_PREFIX = 'folders'

export const listLibraryItemsOptions = (
  api: API,
  query?: LibraryItemListQuery,
) =>
  queryOptions({
    queryKey: [FOLDERS_PREFIX],
    queryFn: () => api.library.list(query ?? {}),
  })
