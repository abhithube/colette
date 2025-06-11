import { getConfig } from '@colette/core/http'
import { queryOptions } from '@tanstack/react-query'

const CONFIG_PREFIX = 'config'

export const getConfigOptions = () =>
  queryOptions({
    queryKey: [CONFIG_PREFIX],
    queryFn: () => getConfig(),
  })
