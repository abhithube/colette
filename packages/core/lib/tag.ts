import { z } from 'zod'
import { type RequestOptions, type UUID, uuidSchema } from './common'

export const tagSchema = z.object({
  id: uuidSchema,
  title: z.string(),
  bookmarkCount: z.number().int().nonnegative().optional(),
  feedCount: z.number().int().nonnegative().optional(),
})

export type Tag = z.infer<typeof tagSchema>

export const tagListSchema = z.object({
  data: tagSchema.array(),
  cursor: z.string().optional(),
})

export type TagList = z.infer<typeof tagListSchema>

export const tagCreateSchema = z.object({
  title: z.string().min(1),
})

export type TagCreate = z.infer<typeof tagCreateSchema>

export const tagUpdateSchema = z.object({
  title: z.string().min(1).optional(),
})

export type TagUpdate = z.infer<typeof tagUpdateSchema>

export const listTagsQuerySchema = z.object({
  tagType: z.enum(['all', 'bookmarks', 'feeds']).optional(),
})

export type ListTagsQuery = z.infer<typeof listTagsQuerySchema>

export interface TagAPI {
  list(query: ListTagsQuery, options?: RequestOptions): Promise<TagList>

  get(id: UUID, options?: RequestOptions): Promise<Tag>

  create(body: TagCreate, options?: RequestOptions): Promise<Tag>

  update(id: UUID, body: TagUpdate, options?: RequestOptions): Promise<Tag>

  delete(id: UUID, options?: RequestOptions): Promise<void>
}
