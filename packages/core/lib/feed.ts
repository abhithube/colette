import { z } from 'zod'
import { type RequestOptions, type UUID, uuidSchema } from './common'
import { tagCreateSchema, tagSchema } from './tag'

export const feedSchema = z.object({
  id: uuidSchema,
  link: z.string().url(),
  title: z.string().nullable(),
  originalTitle: z.string(),
  url: z.string().url().nullable(),
  tags: tagSchema.array().optional(),
  unreadCount: z.number().int().nonnegative().optional(),
})

export type Feed = z.infer<typeof feedSchema>

export const feedListSchema = z.object({
  data: feedSchema.array(),
  cursor: z.string().optional(),
})

export type FeedList = z.infer<typeof feedListSchema>

export const feedCreateSchema = z.object({
  url: z.string().url(),
})

export type FeedCreate = z.infer<typeof feedCreateSchema>

export const feedUpdateSchema = z.object({
  title: z.string().nullable().optional(),
  tags: tagCreateSchema.array().optional(),
})

export type FeedUpdate = z.infer<typeof feedUpdateSchema>

export const fileSchema = z.object({
  data: z.string(),
})

export type File = z.infer<typeof fileSchema>

export const listFeedsQuerySchema = z.object({
  filterByTags: z.boolean().optional(),
  'tag[]': z.string().array().optional(),
})

export type ListFeedsQuery = z.infer<typeof listFeedsQuerySchema>

export interface FeedAPI {
  list(query: ListFeedsQuery, options?: RequestOptions): Promise<FeedList>

  get(id: UUID, options?: RequestOptions): Promise<Feed>

  create(body: FeedCreate, options?: RequestOptions): Promise<Feed>

  update(id: UUID, body: FeedUpdate, options?: RequestOptions): Promise<Feed>

  delete(id: UUID, options?: RequestOptions): Promise<void>

  import(body: File, options?: RequestOptions): Promise<void>
}
