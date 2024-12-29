import type { API } from '@colette/core'
import type { MutationOptions } from '@tanstack/query-core'

export type ImportOpmlOptions = MutationOptions<void, Error, File>

export const importOpmlOptions = (
  options: Omit<ImportOpmlOptions, 'mutationFn'>,
  api: API,
): ImportOpmlOptions => ({
  ...options,
  mutationFn: (body) => api.backups.import(body),
})
