import type { API } from '@colette/core'
import type { BaseMutationOptions } from './common'

type ImportOpmlOptions = BaseMutationOptions<void, File>

export const importOpmlOptions = (
  options: Omit<ImportOpmlOptions, 'mutationFn'>,
  api: API,
): ImportOpmlOptions => ({
  ...options,
  mutationFn: (body) => api.backups.import(body),
})
