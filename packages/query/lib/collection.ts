import {
  createCollection,
  deleteCollection,
  getCollection,
  listCollections,
  updateCollection,
  type CollectionCreate,
  type CollectionUpdate,
} from '@colette/core'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const COLLECTIONS_PREFIX = 'collections'

export const listCollectionsOptions = () =>
  queryOptions({
    queryKey: [COLLECTIONS_PREFIX],
    queryFn: () => listCollections(),
  })

export const getCollectionOptions = (id: string) =>
  queryOptions({
    queryKey: [COLLECTIONS_PREFIX, id],
    queryFn: () => getCollection(id),
  })

export const useCreateCollectionMutation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CollectionCreate) => createCollection(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [COLLECTIONS_PREFIX],
      })
    },
  })
}

export const useUpdateCollectionMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CollectionUpdate) => updateCollection(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [COLLECTIONS_PREFIX],
      })
    },
  })
}

export const useDeleteCollectionMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => deleteCollection(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [COLLECTIONS_PREFIX],
      })
    },
  })
}
