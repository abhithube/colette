import { z } from 'zod'

export type RequestOptions = {
  signal?: AbortSignal | null
}

export const uuidSchema = z.string().uuid()

export type UUID = z.infer<typeof uuidSchema>
