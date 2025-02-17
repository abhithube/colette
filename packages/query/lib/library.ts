import type { API, TreeListQuery } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

const TREE_PREFIX = 'tree'

export const listFeedTreeItemsOptions = (api: API, query?: TreeListQuery) =>
  queryOptions({
    queryKey: [TREE_PREFIX, 'feeds', query],
    queryFn: () => api.library.listFeedTree(query ?? {}),
  })

export const listCollectionTreeItemsOptions = (
  api: API,
  query?: TreeListQuery,
) =>
  queryOptions({
    queryKey: [TREE_PREFIX, 'collections', query],
    queryFn: () => api.library.listCollectionTree(query ?? {}),
  })
