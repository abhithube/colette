import type { API, StreamCreate, StreamUpdate } from '@colette/core'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const STREAMS_PREFIX = 'streams'

export const listStreamsOptions = (api: API) =>
  queryOptions({
    queryKey: [STREAMS_PREFIX],
    queryFn: () => api.streams.listStreams(),
  })

export const getStreamOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [STREAMS_PREFIX, id],
    queryFn: () => api.streams.getStream(id),
  })

export const useCreateStreamMutation = (api: API) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: StreamCreate) => api.streams.createStream(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const useUpdateStreamMutation = (api: API, id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: StreamUpdate) => api.streams.updateStream(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const useDeleteStreamMutation = (api: API, id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.streams.deleteStream(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}
