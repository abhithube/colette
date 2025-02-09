import type { BaseQueryOptions } from './common'
import type { API, LibraryItemList, LibraryItemListQuery } from '@colette/core'

const FOLDERS_PREFIX = 'folders'

type ListLibraryItemsOptions = BaseQueryOptions<LibraryItemList>

export const listLibraryItemsOptions = (
  query: LibraryItemListQuery,
  api: API,
  options: Omit<ListLibraryItemsOptions, 'queryKey' | 'queryFn'> = {},
): ListLibraryItemsOptions => ({
  ...options,
  queryKey: [FOLDERS_PREFIX],
  queryFn: () => api.library.list(query),
})
