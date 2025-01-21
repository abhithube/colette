import type { BaseMutationOptions, BaseQueryOptions } from './common'
import type {
  API,
  Tag,
  TagCreate,
  TagList,
  TagListQuery,
  TagUpdate,
} from '@colette/core'
import { QueryClient } from '@tanstack/query-core'

const TAGS_PREFIX = 'tags'

type ListTagsOptions = BaseQueryOptions<TagList>

export const listTagsOptions = (
  query: TagListQuery,
  api: API,
  options: Omit<ListTagsOptions, 'queryKey' | 'queryFn'> = {},
): ListTagsOptions => ({
  ...options,
  queryKey: [TAGS_PREFIX, query],
  queryFn: () => api.tags.list(query),
})

type GetTagOptions = BaseQueryOptions<Tag>

export const getTagOptions = (
  id: string,
  api: API,
  options: Omit<GetTagOptions, 'queryKey' | 'queryFn'> = {},
): GetTagOptions => ({
  ...options,
  queryKey: [TAGS_PREFIX, id],
  queryFn: () => api.tags.get(id),
})

type CreateTagOptions = BaseMutationOptions<Tag, TagCreate>

export const createTagOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<CreateTagOptions, 'mutationFn'> = {},
): CreateTagOptions => ({
  ...options,
  mutationFn: (body) => api.tags.create(body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [TAGS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

type UpdateTagOptions = BaseMutationOptions<
  Tag,
  { id: string; body: TagUpdate }
>

export const updateTagOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<UpdateTagOptions, 'mutationFn'> = {},
): UpdateTagOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.tags.update(id, body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [TAGS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

export const deleteTagOptions = (
  id: string,
  api: API,
  queryClient: QueryClient,
  options: Omit<BaseMutationOptions, 'mutationFn'> = {},
): BaseMutationOptions => ({
  ...options,
  mutationFn: () => api.tags.delete(id),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [TAGS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})
