import type { API } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

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
