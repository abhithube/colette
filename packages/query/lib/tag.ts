import type {
  API,
  TagCreate,
  TagGetQuery,
  TagListQuery,
  TagUpdate,
} from '@colette/core'
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
    queryFn: () => api.tags.listTags(query),
  })

export const getTagOptions = (api: API, id: string, query: TagGetQuery = {}) =>
  queryOptions({
    queryKey: [TAGS_PREFIX, id],
    queryFn: () => api.tags.getTag(id, query),
  })

export const useCreateTagMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: TagCreate) => api.tags.createTag(data),
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
    mutationFn: (data: TagUpdate) => api.tags.updateTag(id, data),
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
    mutationFn: () => api.tags.deleteTag(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [TAGS_PREFIX],
      })
    },
  })
}
