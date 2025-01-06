import type {
  API,
  Collection,
  CollectionCreate,
  CollectionList,
} from '@colette/core'
import type { MutationOptions, QueryKey } from '@tanstack/query-core'
import type { BaseMutationOptions, BaseQueryOptions } from './common'

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
  options: Omit<CreateCollectionOptions, 'mutationFn'>,
  api: API,
): CreateCollectionOptions => ({
  ...options,
  mutationFn: (body) => api.collections.create(body),
})

export const deleteCollectionOptions = (
  id: string,
  options: Omit<BaseMutationOptions, 'mutationFn'>,
  api: API,
): MutationOptions => ({
  ...options,
  mutationFn: () => api.collections.delete(id),
})
