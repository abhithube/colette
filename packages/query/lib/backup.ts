import { SUBSCRIPTIONS_PREFIX } from './subscription'
import { useAPI } from '@colette/util'
import { useMutation, useQueryClient } from '@tanstack/react-query'

export const useImportOPMLMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: File) => api.backups.importOPML(data),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: [SUBSCRIPTIONS_PREFIX],
      })
    },
  })
}
