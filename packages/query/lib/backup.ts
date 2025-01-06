import type { API } from '@colette/core'
import type { BaseMutationOptions } from './common'

type ImportOpmlOptions = BaseMutationOptions<void, File>

export const importOpmlOptions = (
  api: API,
  options: Omit<ImportOpmlOptions, 'mutationFn'> = {},
): ImportOpmlOptions => ({
  ...options,
  mutationFn: (body) => api.backups.import(body),
})
