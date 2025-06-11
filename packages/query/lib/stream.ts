import {
  createStream,
  deleteStream,
  getStream,
  listStreams,
  updateStream,
} from '@colette/core/http'
import { StreamCreate, StreamUpdate } from '@colette/core/types'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const STREAMS_PREFIX = 'streams'

export const listStreamsOptions = () =>
  queryOptions({
    queryKey: [STREAMS_PREFIX],
    queryFn: () => listStreams(),
  })

export const getStreamOptions = (id: string) =>
  queryOptions({
    queryKey: [STREAMS_PREFIX, id],
    queryFn: () => getStream(id),
  })

export const useCreateStreamMutation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: StreamCreate) => createStream(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const useUpdateStreamMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: StreamUpdate) => updateStream(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const useDeleteStreamMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => deleteStream(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}
