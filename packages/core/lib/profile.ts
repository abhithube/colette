import { z } from 'zod'
import type { RequestOptions } from './common'

export const profileSchema = z.object({
  id: z.string().uuid(),
  title: z.string(),
  imageUrl: z.string().url().nullable(),
  isDefault: z.boolean(),
  userId: z.string().uuid(),
})

export type Profile = z.infer<typeof profileSchema>

export const profileListSchema = z.object({
  data: profileSchema.array(),
  cursor: z.string().optional(),
})

export type ProfileList = z.infer<typeof profileListSchema>

export const profileCreateSchema = z.object({
  title: z.string().min(1),
  imageUrl: z.string().url().nullable().optional(),
})

export type ProfileCreate = z.infer<typeof profileCreateSchema>

export const profileUpdateSchema = z.object({
  title: z.string().min(1).optional(),
  imageUrl: z.string().url().nullable().optional(),
})

export type ProfileUpdate = z.infer<typeof profileUpdateSchema>

export interface ProfileAPI {
  list(options?: RequestOptions): Promise<ProfileList>

  get(id: string, options?: RequestOptions): Promise<Profile>

  getActive(options?: RequestOptions): Promise<Profile>

  create(body: ProfileCreate, options?: RequestOptions): Promise<Profile>

  update(
    id: string,
    body: ProfileUpdate,
    options?: RequestOptions,
  ): Promise<Profile>

  delete(id: string, options?: RequestOptions): Promise<void>
}
