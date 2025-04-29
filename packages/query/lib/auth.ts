import type { API, Login, Register } from '@colette/core'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const AUTH_PREFIX = 'auth'

export const useRegisterUserMutation = (api: API) => {
  return useMutation({
    mutationFn: (data: Register) => api.auth.registerUser(data),
  })
}

export const useLoginUserMutation = (api: API) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: Login) => api.auth.loginUser(data),
    onSuccess: (user) => {
      queryClient.setQueryData([AUTH_PREFIX], user)
    },
  })
}

export const getActiveUserOptions = (api: API) =>
  queryOptions({
    queryKey: [AUTH_PREFIX],
    queryFn: () => api.auth.getActiveUser(),
  })

export const useLogoutUserMutation = (api: API) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => api.auth.logoutUser(),
    onSuccess: () => {
      queryClient.removeQueries({
        queryKey: [AUTH_PREFIX],
        exact: true,
      })
    },
  })
}
