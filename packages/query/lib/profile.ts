import type { API } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

export const listProfilesOptions = (api: API) =>
  queryOptions({
    queryKey: ['profiles'],
    queryFn: ({ signal }) =>
      api.profiles.list({
        signal,
      }),
  })

export const getDefaultProfileOptions = (api: API) =>
  queryOptions({
    queryKey: ['profiles', '@me'],
    queryFn: ({ signal }) =>
      api.profiles.getActive({
        signal,
      }),
  })
