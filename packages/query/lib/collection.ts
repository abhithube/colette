import type {
  API,
  CollectionBookmarkListQuery,
  CollectionCreate,
  CollectionUpdate,
} from '@colette/core'
import { useAPI } from '@colette/util'
import {
  infiniteQueryOptions,
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const COLLECTIONS_PREFIX = 'collections'

export const listCollectionsOptions = (api: API) =>
  queryOptions({
    queryKey: [COLLECTIONS_PREFIX],
    queryFn: () => api.collections.list(),
  })

export const getCollectionOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [COLLECTIONS_PREFIX, id],
    queryFn: () => api.collections.get(id),
  })

export const useCreateCollectionMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CollectionCreate) => api.collections.create(data),
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
    mutationFn: (data: CollectionUpdate) => api.collections.update(id, data),
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
    mutationFn: () => api.collections.delete(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [COLLECTIONS_PREFIX],
      })
    },
  })
}

export const listCollectionBookmarksOptions = (
  api: API,
  id: string,
  query: Omit<CollectionBookmarkListQuery, 'cursor'> = {},
) =>
  infiniteQueryOptions({
    queryKey: [COLLECTIONS_PREFIX, id, 'bookmarks', query],
    queryFn: ({ pageParam }) =>
      api.collections.listBookmarks(id, {
        ...query,
        cursor: pageParam,
      }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.cursor,
  })
