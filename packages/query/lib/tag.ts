import type { API, ListTagsQuery, Tag, TagCreate } from '@colette/core'
import { type UseMutationOptions, queryOptions } from '@tanstack/react-query'

export const listTagsOptions = (
  query: ListTagsQuery,
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

export type CreateTagOptions = UseMutationOptions<Tag, Error, TagCreate>

export const createTagOptions = (
  options: Omit<CreateTagOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.tags.create(body),
  } as CreateTagOptions
}
