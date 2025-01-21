import type { BaseMutationOptions, BaseQueryOptions } from './common'
import type { API, Login, User } from '@colette/core'
import type { QueryClient } from '@tanstack/query-core'

const AUTH_PREFIX = 'auth'

type LoginOptions = BaseMutationOptions<User, Login>

export const loginOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<LoginOptions, 'mutationFn'> = {},
): LoginOptions => ({
  ...options,
  mutationFn: (body) => api.auth.login(body),
  onSuccess: async (...args) => {
    queryClient.setQueryData([AUTH_PREFIX], args[0])

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

type GetActiveOptions = BaseQueryOptions<User>

export const getActiveOptions = (
  api: API,
  options: Omit<GetActiveOptions, 'queryKey' | 'queryFn'> = {},
): GetActiveOptions => ({
  ...options,
  queryKey: [AUTH_PREFIX],
  queryFn: () => api.auth.getActive(),
})

type LogoutOptions = BaseMutationOptions<void, void>

export const logoutOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<LogoutOptions, 'mutationFn'> = {},
): LogoutOptions => ({
  ...options,
  mutationFn: () => api.auth.logout(),
  onSuccess: async (...args) => {
    queryClient.removeQueries({
      queryKey: [AUTH_PREFIX],
      exact: true,
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})
