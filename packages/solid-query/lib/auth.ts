import type { API, Login, Profile, SwitchProfile } from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'

export type LoginOptions = MutationOptions<Profile, Error, Login>

export const loginOptions = (
  options: Omit<LoginOptions, 'mutationFn'>,
  api: API,
): LoginOptions => ({
  ...options,
  mutationFn: (body) => api.auth.login(body),
})

export type SwitchProfileOptions = MutationOptions<
  Profile,
  Error,
  SwitchProfile
>

export const switchProfileOptions = (
  options: Omit<SwitchProfileOptions, 'mutationFn'>,
  api: API,
): SwitchProfileOptions => ({
  ...options,
  mutationFn: (body) => api.auth.switchProfile(body),
})
