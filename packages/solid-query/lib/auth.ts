import type { API, Login, User } from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'

export type LoginOptions = MutationOptions<User, Error, Login>

export const loginOptions = (
  options: Omit<LoginOptions, 'mutationFn'>,
  api: API,
): LoginOptions => ({
  ...options,
  mutationFn: (body) => api.auth.login(body),
})
