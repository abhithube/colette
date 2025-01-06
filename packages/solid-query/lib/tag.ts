import type { API, Tag, TagCreate, TagList, TagListQuery } from '@colette/core'
import type { QueryKey } from '@tanstack/query-core'
import type { BaseMutationOptions, BaseQueryOptions } from './common'

const TAGS_KEY: QueryKey = ['tags']

type ListTagsOptions = BaseQueryOptions<TagList>

export const listTagsOptions = (
  query: TagListQuery,
  api: API,
  options: Omit<ListTagsOptions, 'queryKey' | 'queryFn'> = {},
): ListTagsOptions => ({
  ...options,
  queryKey: [...TAGS_KEY, query],
  queryFn: () => api.tags.list(query),
})

type GetTagOptions = BaseQueryOptions<Tag>

export const getTagOptions = (
  id: string,
  api: API,
  options: Omit<GetTagOptions, 'queryKey' | 'queryFn'> = {},
): GetTagOptions => ({
  ...options,
  queryKey: [...TAGS_KEY, id],
  queryFn: () => api.tags.get(id),
})

type CreateTagOptions = BaseMutationOptions<Tag, TagCreate>

export const createTagOptions = (
  api: API,
  options: Omit<CreateTagOptions, 'mutationFn'> = {},
): CreateTagOptions => ({
  ...options,
  mutationFn: (body) => api.tags.create(body),
})
