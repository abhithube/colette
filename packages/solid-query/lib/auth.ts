import type { API, Login, User } from '@colette/core'
import type {
  MutationOptions,
  QueryClient,
  QueryKey,
} from '@tanstack/query-core'
import { queryOptions } from '@tanstack/solid-query'

const authQueryKey = () => ['auth'] as QueryKey

export type LoginOptions = MutationOptions<User, Error, Login>

export const loginOptions = (
  options: Omit<LoginOptions, 'mutationFn'>,
  api: API,
  queryClient: QueryClient,
): LoginOptions => ({
  ...options,
  mutationFn: (body) => api.auth.login(body),
  onSuccess: async (...args) => {
    queryClient.setQueryData(authQueryKey(), args[0])

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})

export const getActiveOptions = (api: API) =>
  queryOptions({
    queryKey: authQueryKey(),
    queryFn: () => api.auth.getActive(),
  })

export const logoutOptions = (
  options: Omit<MutationOptions<void, Error, void>, 'mutationFn'>,
  api: API,
  queryClient: QueryClient,
): MutationOptions<void, Error, void> => ({
  ...options,
  mutationFn: () => api.auth.logout(),
  onSuccess: async (...args) => {
    queryClient.setQueryData(authQueryKey(), null)
    await queryClient.invalidateQueries({
      queryKey: authQueryKey(),
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})
