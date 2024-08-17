import type { API, Login, Profile } from '@colette/core'
import type { UseMutationOptions } from '@tanstack/react-query'

export const loginOptions = (
  options: Omit<UseMutationOptions<Profile, Error, Login>, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.auth.login(body),
  } as UseMutationOptions<Profile, Error, Login>
}
