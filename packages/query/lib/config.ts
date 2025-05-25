import type { API } from '@colette/core'
import { queryOptions } from '@tanstack/react-query'

const CONFIG_PREFIX = 'config'

export const getConfigOptions = (api: API) =>
  queryOptions({
    queryKey: [CONFIG_PREFIX],
    queryFn: () => api.config.getConfig(),
  })
