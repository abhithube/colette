import { getActiveUser } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

const AUTH_PREFIX = 'auth'

export const getActiveUserOptions = () =>
  queryOptions({
    queryKey: [AUTH_PREFIX],
    queryFn: () => getActiveUser(),
  })
