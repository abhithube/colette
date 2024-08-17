import { z } from 'zod'
import { type RequestOptions, type UUID, uuidSchema } from './common'
import { tagCreateSchema, tagSchema } from './tag'

export const bookmarkSchema = z.object({
  id: uuidSchema,
  link: z.string().url(),
  title: z.string(),
  thumbnailUrl: z.string().url().nullable(),
  publishedAt: z.string().datetime().nullable(),
  author: z.string().nullable(),
  collectionId: uuidSchema.nullable(),
  sortIndex: z.number().nonnegative(),
  tags: tagSchema.array().optional(),
})

export type Bookmark = z.infer<typeof bookmarkSchema>

export const bookmarkListSchema = z.object({
  data: bookmarkSchema.array(),
  cursor: z.string().optional(),
})

export type BookmarkList = z.infer<typeof bookmarkListSchema>

export const bookmarkCreateSchema = z.object({
  url: z.string().url(),
  collectionId: uuidSchema.nullable().optional(),
})

export type BookmarkCreate = z.infer<typeof bookmarkCreateSchema>

export const bookmarkUpdateSchema = z.object({
  sortIndex: z.number().optional(),
  collectionId: uuidSchema.nullable().optional(),
  tags: tagCreateSchema.array().optional(),
})

export type BookmarkUpdate = z.infer<typeof bookmarkUpdateSchema>

export const listBookmarksQuerySchema = z.object({
  filterByCollection: z.boolean().optional(),
  collectionId: uuidSchema.optional(),
  filterByTags: z.boolean().optional(),
  'tag[]': z.string().array().optional(),
  cursor: z.string().optional(),
})

export type ListBookmarksQuery = z.infer<typeof listBookmarksQuerySchema>

export interface BookmarkAPI {
  list(
    query: ListBookmarksQuery,
    options?: RequestOptions,
  ): Promise<BookmarkList>

  get(id: UUID, options?: RequestOptions): Promise<Bookmark>

  create(body: BookmarkCreate, options?: RequestOptions): Promise<Bookmark>

  update(
    id: UUID,
    body: BookmarkUpdate,
    options?: RequestOptions,
  ): Promise<Bookmark>

  delete(id: UUID, options?: RequestOptions): Promise<void>
}
