import type { API } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

const AUTH_PREFIX = 'auth'

export const getActiveUserOptions = (api: API) =>
  queryOptions({
    queryKey: [AUTH_PREFIX],
    queryFn: () => api.auth.getActiveUser(),
  })
