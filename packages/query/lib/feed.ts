import type {
  API,
  FeedCreate,
  FeedDetect,
  FeedListQuery,
  FeedUpdate,
} from '@colette/core'
import { useAPI } from '@colette/util'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

export const FEEDS_PREFIX = 'feeds'

export const listFeedsOptions = (api: API, query: FeedListQuery = {}) =>
  queryOptions({
    queryKey: [FEEDS_PREFIX, query],
    queryFn: () => api.feeds.list(query),
  })

export const getFeedOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [FEEDS_PREFIX, id],
    queryFn: () => api.feeds.get(id),
  })

export const useCreateFeedMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: FeedCreate) => api.feeds.create(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [FEEDS_PREFIX],
      })
    },
  })
}

export const useUpdateFeedMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: FeedUpdate) => api.feeds.update(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [FEEDS_PREFIX],
      })
    },
  })
}

export const useDeleteFeedMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.feeds.delete(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [FEEDS_PREFIX],
      })
    },
  })
}

export const useDetectFeedsMutation = () => {
  const api = useAPI()

  return useMutation({
    mutationFn: (data: FeedDetect) => api.feeds.detect(data),
  })
}
