import type { BaseMutationOptions, BaseQueryOptions } from './common'
import type {
  API,
  Folder,
  FolderCreate,
  FolderList,
  FolderListQuery,
  FolderUpdate,
} from '@colette/core'
import type { QueryClient } from '@tanstack/query-core'

const FOLDERS_PREFIX = 'Folders'

type ListFoldersOptions = BaseQueryOptions<FolderList>

export const listFoldersOptions = (
  query: FolderListQuery,
  api: API,
  options: Omit<ListFoldersOptions, 'queryKey' | 'queryFn'> = {},
): ListFoldersOptions => ({
  ...options,
  queryKey: [FOLDERS_PREFIX],
  queryFn: () => api.folders.list(query),
})

type GetFolderOptions = BaseQueryOptions<Folder>

export const getFolderOptions = (
  id: string,
  api: API,
  options: Omit<GetFolderOptions, 'queryKey' | 'queryFn'> = {},
): GetFolderOptions => ({
  ...options,
  queryKey: [FOLDERS_PREFIX, id],
  queryFn: () => api.folders.get(id),
})

type CreateFolderOptions = BaseMutationOptions<Folder, FolderCreate>

export const createFolderOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<CreateFolderOptions, 'mutationFn'> = {},
): CreateFolderOptions => ({
  ...options,
  mutationFn: (body) => api.folders.create(body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [FOLDERS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

type UpdateFolderOptions = BaseMutationOptions<
  Folder,
  { id: string; body: FolderUpdate }
>

export const updateFolderOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<UpdateFolderOptions, 'mutationFn'> = {},
): UpdateFolderOptions => ({
  ...options,
  mutationFn: ({ id, body }) => api.folders.update(id, body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [FOLDERS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

export const deleteFolderOptions = (
  id: string,
  api: API,
  queryClient: QueryClient,
  options: Omit<BaseMutationOptions, 'mutationFn'> = {},
): BaseMutationOptions => ({
  ...options,
  mutationFn: () => api.folders.delete(id),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [FOLDERS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})
