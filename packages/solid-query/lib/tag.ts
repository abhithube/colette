import type { API, Tag, TagCreate, TagListQuery } from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'
import { queryOptions } from '@tanstack/solid-query'

export const listTagsOptions = (
  query: TagListQuery,
  profileId: string,
  api: API,
) =>
  queryOptions({
    queryKey: ['profiles', profileId, 'tags', query],
    queryFn: () => api.tags.list(query),
  })

export const getTagOptions = (id: string, api: API) =>
  queryOptions({
    queryKey: ['tags', id],
    queryFn: () => api.tags.get(id),
  })

export type CreateTagOptions = MutationOptions<Tag, Error, TagCreate>

export const createTagOptions = (
  options: Omit<CreateTagOptions, 'mutationFn'>,
  api: API,
): CreateTagOptions => ({
  ...options,
  mutationFn: (body) => api.tags.create(body),
})
