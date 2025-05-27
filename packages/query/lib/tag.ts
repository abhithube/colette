import {
  type TagCreate,
  type GetTagQueryParams,
  type ListTagsQueryParams,
  type TagUpdate,
  listTags,
  getTag,
  createTag,
  updateTag,
  deleteTag,
} from '@colette/core'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const TAGS_PREFIX = 'tags'

export const listTagsOptions = (query: ListTagsQueryParams = {}) =>
  queryOptions({
    queryKey: [TAGS_PREFIX, query],
    queryFn: () => listTags(query),
  })

export const getTagOptions = (id: string, query: GetTagQueryParams = {}) =>
  queryOptions({
    queryKey: [TAGS_PREFIX, id],
    queryFn: () => getTag(id, query),
  })

export const useCreateTagMutation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: TagCreate) => createTag(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [TAGS_PREFIX],
      })
    },
  })
}

export const useUpdateTagMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: TagUpdate) => updateTag(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [TAGS_PREFIX],
      })
    },
  })
}

export const useDeleteTagMutation = (id: string) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => deleteTag(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [TAGS_PREFIX],
      })
    },
  })
}
