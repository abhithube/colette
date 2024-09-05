import type { API } from '@colette/core'
import type { UseMutationOptions } from '@tanstack/react-query'

export type ImportOpmlOptions = UseMutationOptions<void, Error, File>

export const importOpmlOptions = (
  options: Omit<ImportOpmlOptions, 'mutationFn'>,
  api: API,
) => {
  return {
    ...options,
    mutationFn: (body) => api.backups.import(body),
  } as ImportOpmlOptions
}
