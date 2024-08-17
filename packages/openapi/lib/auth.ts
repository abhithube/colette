import {
  APIError,
  type AuthAPI,
  ConflictError,
  type Login,
  type Profile,
  type Register,
  type RequestOptions,
  UnauthorizedError,
  UnprocessableContentError,
  type User,
  loginSchema,
  profileSchema,
  registerSchema,
  userSchema,
} from '@colette/core'
import type { Client } from '.'

export class HTTPAuthAPI implements AuthAPI {
  constructor(private client: Client) {}

  async register(body: Register, options: RequestOptions): Promise<User> {
    const bodyResult = await registerSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.POST('/auth/register', {
      body: bodyResult.data,
      ...options,
    })
    if (res.error) {
      if (res.response.status === 401) {
        throw new UnauthorizedError(res.error.message)
      }
      if (res.response.status === 409) {
        throw new ConflictError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    const userResult = await userSchema.safeParseAsync(res.data)
    if (userResult.error) {
      throw new UnprocessableContentError(userResult.error.message)
    }

    return userResult.data
  }

  async login(body: Login, options?: RequestOptions): Promise<Profile> {
    const bodyResult = await loginSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.POST('/auth/login', {
      body: bodyResult.data,
      ...options,
    })
    if (res.error) {
      if (res.response.status === 401) {
        throw new UnauthorizedError(res.error.message)
      }
      if (res.response.status === 422) {
        throw new UnprocessableContentError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    const profileResult = await profileSchema.safeParseAsync(res.data)
    if (profileResult.error) {
      throw new UnprocessableContentError(profileResult.error.message)
    }

    return profileResult.data
  }

  async getActive(options?: RequestOptions): Promise<User> {
    const res = await this.client.GET('/auth/@me', options)
    if (res.error) {
      throw new APIError('unknown error')
    }

    const userResult = await userSchema.safeParseAsync(res.data)
    if (userResult.error) {
      throw new UnprocessableContentError(userResult.error.message)
    }

    return userResult.data
  }
}
