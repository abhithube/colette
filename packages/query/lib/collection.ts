import type { API, CollectionCreate, CollectionUpdate } from '@colette/core'
import { useAPI } from '@colette/util'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const COLLECTIONS_PREFIX = 'collections'

export const listCollectionsOptions = (api: API) =>
  queryOptions({
    queryKey: [COLLECTIONS_PREFIX],
    queryFn: () => api.collections.listCollections(),
  })

export const getCollectionOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [COLLECTIONS_PREFIX, id],
    queryFn: () => api.collections.getCollection(id),
  })

export const useCreateCollectionMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CollectionCreate) =>
      api.collections.createCollection(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [COLLECTIONS_PREFIX],
      })
    },
  })
}

export const useUpdateCollectionMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CollectionUpdate) =>
      api.collections.updateCollection(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [COLLECTIONS_PREFIX],
      })
    },
  })
}

export const useDeleteCollectionMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.collections.deleteCollection(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [COLLECTIONS_PREFIX],
      })
    },
  })
}
