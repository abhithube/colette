import {
  getActiveUser,
  registerUser,
  loginUser,
  logoutUser,
} from '@colette/core/http'
import type { LoginPayload, RegisterPayload } from '@colette/core/types'
import {
  queryOptions,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query'

const AUTH_PREFIX = 'auth'

export const useRegisterUserMutation = () => {
  return useMutation({
    mutationFn: (data: RegisterPayload) => registerUser(data),
  })
}

export const useLoginUserMutation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: LoginPayload) => loginUser(data),
    onSuccess: (user) => {
      queryClient.setQueryData([AUTH_PREFIX], user)
    },
  })
}

export const getActiveUserOptions = () =>
  queryOptions({
    queryKey: [AUTH_PREFIX],
    queryFn: () => getActiveUser(),
  })

export const useLogoutUserMutation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => logoutUser(),
    onSuccess: () => {
      queryClient.removeQueries({
        queryKey: [AUTH_PREFIX],
        exact: true,
      })
    },
  })
}
