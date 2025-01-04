import type { API, Collection, CollectionCreate } from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'
import { queryOptions } from '@tanstack/solid-query'

export const listCollectionsOptions = (api: API) =>
  queryOptions({
    queryKey: ['collections'],
    queryFn: () => api.collections.list(),
  })

export const getCollectionOptions = (id: string, api: API) =>
  queryOptions({
    queryKey: ['collections', id],
    queryFn: () => api.collections.get(id),
  })

export type CreateCollectionOptions = MutationOptions<
  Collection,
  Error,
  CollectionCreate
>

export const createCollectionOptions = (
  options: Omit<CreateCollectionOptions, 'mutationFn'>,
  api: API,
): CreateCollectionOptions => ({
  ...options,
  mutationFn: (body) => api.collections.create(body),
})

export const deleteCollectionOptions = (
  id: string,
  options: Omit<MutationOptions, 'mutationFn'>,
  api: API,
): MutationOptions => ({
  ...options,
  mutationFn: () => api.collections.delete(id),
})
