import { z } from 'zod'
import type { RequestOptions } from './common'

export const feedEntrySchema = z.object({
  id: z.string().uuid(),
  link: z.string().url(),
  title: z.string(),
  publishedAt: z.string().datetime().nullable(),
  description: z.string().nullable(),
  author: z.string().nullable(),
  thumbnailUrl: z.string().url().nullable(),
  hasRead: z.boolean(),
  feedId: z.string().uuid(),
})

export type FeedEntry = z.infer<typeof feedEntrySchema>

export const feedEntryListSchema = z.object({
  data: feedEntrySchema.array(),
  cursor: z.string().optional(),
})

export type FeedEntryList = z.infer<typeof feedEntryListSchema>

export const feedEntryUpdateSchema = z.object({
  hasRead: z.boolean().optional(),
})

export type FeedEntryUpdate = z.infer<typeof feedEntryUpdateSchema>

export const listFeedEntriesQuerySchema = z.object({
  feedId: z.string().uuid().optional(),
  hasRead: z.boolean().optional(),
  'tag[]': z.string().array().optional(),
  cursor: z.string().optional(),
})

export type ListFeedEntriesQuery = z.infer<typeof listFeedEntriesQuerySchema>

export interface FeedEntryAPI {
  list(
    query: ListFeedEntriesQuery,
    options?: RequestOptions,
  ): Promise<FeedEntryList>

  update(
    id: string,
    body: FeedEntryUpdate,
    options?: RequestOptions,
  ): Promise<FeedEntry>
}
