import { FEEDS_PREFIX } from './feed'
import { useAPI } from '@colette/util'
import { useMutation, useQueryClient } from '@tanstack/react-query'

export const useImportOPMLMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: File) => api.backups.import(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [FEEDS_PREFIX],
      })
    },
  })
}
