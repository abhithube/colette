import type {
  API,
  FolderCreate,
  FolderListQuery,
  FolderUpdate,
} from '@colette/core'
import { useAPI } from '@colette/util'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const FOLDERS_PREFIX = 'Folders'

export const listFoldersOptions = (api: API, query?: FolderListQuery) =>
  queryOptions({
    queryKey: [FOLDERS_PREFIX],
    queryFn: () => api.folders.list(query ?? {}),
  })

export const getFolderOptions = (api: API, id: string) =>
  queryOptions({
    queryKey: [FOLDERS_PREFIX, id],
    queryFn: () => api.folders.get(id),
  })

export const useCreateFolderMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: FolderCreate) => api.folders.create(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [FOLDERS_PREFIX],
      })
    },
  })
}

export const useUpdateFolderMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: FolderUpdate) => api.folders.update(id, data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [FOLDERS_PREFIX],
      })
    },
  })
}

export const useDeleteFolderMutation = (id: string) => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.folders.delete(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [FOLDERS_PREFIX],
      })
    },
  })
}
