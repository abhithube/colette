import type { API, Login, Profile, SwitchProfile } from '@colette/core'
import type { UseMutationOptions } from '@tanstack/react-query'

export type LoginOptions = UseMutationOptions<Profile, Error, Login>

export const loginOptions = (
  options: Omit<LoginOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.auth.login(body),
  } as LoginOptions
}

export type SwitchProfileOptions = UseMutationOptions<
  Profile,
  Error,
  SwitchProfile
>

export const switchProfileOptions = (
  options: Omit<SwitchProfileOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.auth.switchProfile(body),
  } as SwitchProfileOptions
}
