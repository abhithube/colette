import type { API } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

export const listSmartFeedsOptions = (profileId: string, api: API) =>
  queryOptions({
    queryKey: ['profiles', profileId, 'smartFeeds'],
    queryFn: () => api.smartFeeds.list(),
  })

export const getSmartFeedOptions = (id: string, api: API) =>
  queryOptions({
    queryKey: ['smartFeeds', id],
    queryFn: () => api.smartFeeds.get(id),
  })
