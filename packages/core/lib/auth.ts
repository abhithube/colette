import { z } from 'zod'
import { type RequestOptions, uuidSchema } from './common'
import type { Profile } from './profile'

export const userSchema = z.object({
  id: uuidSchema,
  email: z.string().email(),
})

export type User = z.infer<typeof userSchema>

export const registerSchema = z.object({
  email: z.string().email(),
  password: z.string(),
})

export type Register = z.infer<typeof registerSchema>

export const loginSchema = z.object({
  email: z.string().email(),
  password: z.string(),
})

export type Login = z.infer<typeof loginSchema>

export interface AuthAPI {
  register(body: Register, options?: RequestOptions): Promise<User>

  login(body: Login, options?: RequestOptions): Promise<Profile>

  getActive(options?: RequestOptions): Promise<User>
}
