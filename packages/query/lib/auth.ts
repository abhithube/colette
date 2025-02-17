import type { API, Login } from '@colette/core'
import { useAPI } from '@colette/util'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const AUTH_PREFIX = 'auth'

export const useLoginMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: Login) => api.auth.login(data),
    onSuccess: (user) => {
      queryClient.setQueryData([AUTH_PREFIX], user)
    },
  })
}

export const getActiveOptions = (api: API) =>
  queryOptions({
    queryKey: [AUTH_PREFIX],
    queryFn: () => api.auth.getActive(),
  })

export const useLogoutMutation = () => {
  const api = useAPI()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.auth.logout(),
    onSuccess: () => {
      queryClient.removeQueries({
        queryKey: [AUTH_PREFIX],
        exact: true,
      })
    },
  })
}
