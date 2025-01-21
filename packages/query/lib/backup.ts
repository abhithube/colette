import type { BaseMutationOptions } from './common'
import { FEEDS_PREFIX } from './feed'
import type { API } from '@colette/core'
import { QueryClient } from '@tanstack/query-core'

type ImportOpmlOptions = BaseMutationOptions<void, File>

export const importOpmlOptions = (
  api: API,
  queryClient: QueryClient,
  options: Omit<ImportOpmlOptions, 'mutationFn'> = {},
): ImportOpmlOptions => ({
  ...options,
  mutationFn: (body) => api.backups.import(body),
  onSuccess: async (...args) => {
    await queryClient.invalidateQueries({
      queryKey: [FEEDS_PREFIX],
    })

    if (options.onSuccess) {
      await options.onSuccess(...args)
    }
  },
})
