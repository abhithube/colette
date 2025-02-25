import type { API, StreamCreate, StreamUpdate } from '@colette/core'
import { useAPI } from '@colette/util'
import {
  infiniteQueryOptions,
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const STREAMS_PREFIX = 'streams'

export const listStreamsOptions = (api: API) =>
  queryOptions({
    queryKey: [STREAMS_PREFIX],
    queryFn: () => api.streams.list(),
  })

export const getStreamOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [STREAMS_PREFIX, id],
    queryFn: () => api.streams.get(id),
  })

export const useCreateStreamMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: StreamCreate) => api.streams.create(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const useUpdateStreamMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: StreamUpdate) => api.streams.update(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const useDeleteStreamMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.streams.delete(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const listStreamEntriesOptions = (api: API, id: string) =>
  infiniteQueryOptions({
    queryKey: [STREAMS_PREFIX, id, 'entries'],
    queryFn: () => api.streams.listEntries(id),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })
