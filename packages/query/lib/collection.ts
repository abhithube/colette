// import type { BaseMutationOptions, BaseQueryOptions } from './common'
// import type {
//   API,
//   Collection,
//   CollectionCreate,
//   CollectionList,
//   CollectionUpdate,
// } from '@colette/core'
// import type { QueryClient } from '@tanstack/query-core'

// const COLLECTIONS_PREFIX = 'collections'

// type ListCollectionsOptions = BaseQueryOptions<CollectionList>

// export const listCollectionsOptions = (
//   api: API,
//   options: Omit<ListCollectionsOptions, 'queryKey' | 'queryFn'> = {},
// ): ListCollectionsOptions => ({
//   ...options,
//   queryKey: [COLLECTIONS_PREFIX],
//   queryFn: () => api.collections.list(),
// })

// type GetCollectionOptions = BaseQueryOptions<Collection>

// export const getCollectionOptions = (
//   id: string,
//   api: API,
//   options: Omit<GetCollectionOptions, 'queryKey' | 'queryFn'> = {},
// ): GetCollectionOptions => ({
//   ...options,
//   queryKey: [COLLECTIONS_PREFIX, id],
//   queryFn: () => api.collections.get(id),
// })

// type CreateCollectionOptions = BaseMutationOptions<Collection, CollectionCreate>

// export const createCollectionOptions = (
//   api: API,
//   queryClient: QueryClient,
//   options: Omit<CreateCollectionOptions, 'mutationFn'> = {},
// ): CreateCollectionOptions => ({
//   ...options,
//   mutationFn: (body) => api.collections.create(body),
//   onSuccess: async (...args) => {
//     await queryClient.invalidateQueries({
//       queryKey: [COLLECTIONS_PREFIX],
//     })

//     if (options.onSuccess) {
//       await options.onSuccess(...args)
//     }
//   },
// })

// type UpdateCollectionOptions = BaseMutationOptions<
//   Collection,
//   { id: string; body: CollectionUpdate }
// >

// export const updateCollectionOptions = (
//   api: API,
//   queryClient: QueryClient,
//   options: Omit<UpdateCollectionOptions, 'mutationFn'> = {},
// ): UpdateCollectionOptions => ({
//   ...options,
//   mutationFn: ({ id, body }) => api.collections.update(id, body),
//   onSuccess: async (...args) => {
//     await queryClient.invalidateQueries({
//       queryKey: [COLLECTIONS_PREFIX],
//     })

//     if (options.onSuccess) {
//       await options.onSuccess(...args)
//     }
//   },
// })

// export const deleteCollectionOptions = (
//   id: string,
//   api: API,
//   queryClient: QueryClient,
//   options: Omit<BaseMutationOptions, 'mutationFn'> = {},
// ): BaseMutationOptions => ({
//   ...options,
//   mutationFn: () => api.collections.delete(id),
//   onSuccess: async (...args) => {
//     await queryClient.invalidateQueries({
//       queryKey: [COLLECTIONS_PREFIX],
//     })

//     if (options.onSuccess) {
//       await options.onSuccess(...args)
//     }
//   },
// })
