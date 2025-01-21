import type { BaseMutationOptions, BaseQueryOptions } from './common'
import type {
  API,
  Collection,
  CollectionCreate,
  CollectionList,
} from '@colette/core'
import type { MutationOptions, QueryKey } from '@tanstack/query-core'

const COLLECTIONS_KEY: QueryKey = ['collections']

type ListCollectionsOptions = BaseQueryOptions<CollectionList>

export const listCollectionsOptions = (
  api: API,
  options: Omit<ListCollectionsOptions, 'queryKey' | 'queryFn'> = {},
): ListCollectionsOptions => ({
  ...options,
  queryKey: COLLECTIONS_KEY,
  queryFn: () => api.collections.list(),
})

type GetCollectionOptions = BaseQueryOptions<Collection>

export const getCollectionOptions = (
  id: string,
  api: API,
  options: Omit<GetCollectionOptions, 'queryKey' | 'queryFn'> = {},
): GetCollectionOptions => ({
  ...options,
  queryKey: [...COLLECTIONS_KEY, id],
  queryFn: () => api.collections.get(id),
})

type CreateCollectionOptions = BaseMutationOptions<Collection, CollectionCreate>

export const createCollectionOptions = (
  api: API,
  options: Omit<CreateCollectionOptions, 'mutationFn'> = {},
): CreateCollectionOptions => ({
  ...options,
  mutationFn: (body) => api.collections.create(body),
})

export const deleteCollectionOptions = (
  id: string,
  api: API,
  options: Omit<BaseMutationOptions, 'mutationFn'> = {},
): MutationOptions => ({
  ...options,
  mutationFn: () => api.collections.delete(id),
})
