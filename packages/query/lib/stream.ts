import type { API, StreamCreate, StreamUpdate } from '@colette/core'
import { useAPI } from '@colette/util'
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

export const useCreateStreamMutation = () => {
  const api = useAPI()
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

export const useUpdateStreamMutation = (id: string) => {
  const api = useAPI()
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

export const useDeleteStreamMutation = (id: string) => {
  const api = useAPI()
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
