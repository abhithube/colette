import type { API, TagCreate, TagListQuery, TagUpdate } from '@colette/core'
import { useAPI } from '@colette/util'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const TAGS_PREFIX = 'tags'

export const listTagsOptions = (api: API, query: TagListQuery = {}) =>
  queryOptions({
    queryKey: [TAGS_PREFIX, query],
    queryFn: () => api.tags.list(query),
  })

export const getTagOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [TAGS_PREFIX, id],
    queryFn: () => api.tags.get(id),
  })

export const useCreateTagMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: TagCreate) => api.tags.create(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [TAGS_PREFIX],
      })
    },
  })
}

export const useUpdateTagMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: TagUpdate) => api.tags.update(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [TAGS_PREFIX],
      })
    },
  })
}

export const useDeleteTagMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.tags.delete(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [TAGS_PREFIX],
      })
    },
  })
}
