import { getActiveUser } from '@colette/core/http'
import { queryOptions } from '@tanstack/react-query'

const AUTH_PREFIX = 'auth'

export const getActiveUserOptions = () =>
  queryOptions({
    queryKey: [AUTH_PREFIX],
    queryFn: () => getActiveUser(),
  })
