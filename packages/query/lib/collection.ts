import type { API, CollectionCreate, CollectionUpdate } from '@colette/core'
import { useAPI } from '@colette/util'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const STREAMS_PREFIX = 'collections'

export const listCollectionsOptions = (api: API) =>
  queryOptions({
    queryKey: [STREAMS_PREFIX],
    queryFn: () => api.collections.list(),
  })

export const getCollectionOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [STREAMS_PREFIX, id],
    queryFn: () => api.collections.get(id),
  })

export const useCreateCollectionMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CollectionCreate) => api.collections.create(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const useUpdateCollectionMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CollectionUpdate) => api.collections.update(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}

export const useDeleteCollectionMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.collections.delete(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [STREAMS_PREFIX],
      })
    },
  })
}
