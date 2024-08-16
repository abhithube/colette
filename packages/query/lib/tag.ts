import type { API, ListTagsQuery, Tag, TagCreate } from '@colette/openapi'
import { type UseMutationOptions, queryOptions } from '@tanstack/react-query'

export const listTagsOptions = (
  query: ListTagsQuery,
  profileId: string,
  api: API,
) =>
  queryOptions({
    queryKey: ['profiles', profileId, 'tags', query],
    queryFn: ({ signal }) =>
      api.tags.list(query, {
        signal,
      }),
  })

export const getTagOptions = (id: string, api: API) =>
  queryOptions({
    queryKey: ['tags', id],
    queryFn: ({ signal }) =>
      api.tags.get(id, {
        signal,
      }),
  })

export const createTagOptions = (
  options: Omit<UseMutationOptions<Tag, Error, TagCreate>, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.tags.create(body),
  } as UseMutationOptions<Tag, Error, TagCreate>
}