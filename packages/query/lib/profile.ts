import type { API, Profile, ProfileCreate } from '@colette/core'
import { type UseMutationOptions, queryOptions } from '@tanstack/react-query'

export const listProfilesOptions = (api: API) =>
  queryOptions({
    queryKey: ['profiles'],
    queryFn: () => api.profiles.list(),
  })

export const getActiveProfileOptions = (api: API) =>
  queryOptions({
    queryKey: ['profiles', '@me'],
    queryFn: () => api.profiles.getActive(),
  })

export type CreateProfileOptions = UseMutationOptions<
  Profile,
  Error,
  ProfileCreate
>

export const createProfileOptions = (
  options: Omit<CreateProfileOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.profiles.create(body),
  } as CreateProfileOptions
}
