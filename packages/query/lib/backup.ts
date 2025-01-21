import type { BaseMutationOptions } from './common'
import type { API } from '@colette/core'

type ImportOpmlOptions = BaseMutationOptions<void, File>

export const importOpmlOptions = (
  api: API,
  options: Omit<ImportOpmlOptions, 'mutationFn'> = {},
): ImportOpmlOptions => ({
  ...options,
  mutationFn: (body) => api.backups.import(body),
})
