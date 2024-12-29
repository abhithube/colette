import type { API, Profile, ProfileCreate } from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'
import { queryOptions } from '@tanstack/solid-query'

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

export type CreateProfileOptions = MutationOptions<
  Profile,
  Error,
  ProfileCreate
>

export const createProfileOptions = (
  options: Omit<CreateProfileOptions, 'mutationFn'>,
  api: API,
): CreateProfileOptions => ({
  ...options,
  mutationFn: (body) => api.profiles.create(body),
})
