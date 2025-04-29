import type { API, CollectionCreate, CollectionUpdate } from '@colette/core'
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

export const useCreateCollectionMutation = (api: API) => {
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

export const useUpdateCollectionMutation = (api: API, id: string) => {
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

export const useDeleteCollectionMutation = (api: API, id: string) => {
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
