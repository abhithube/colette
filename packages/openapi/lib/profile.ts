import {
  APIError,
  ConflictError,
  NotFoundError,
  type Profile,
  type ProfileAPI,
  type ProfileCreate,
  type ProfileList,
  type ProfileUpdate,
  type RequestOptions,
  type UUID,
  UnprocessableContentError,
  profileCreateSchema,
  profileListSchema,
  profileSchema,
  profileUpdateSchema,
  uuidSchema,
} from '@colette/core'
import type { Client } from '.'

export class HTTPProfileAPI implements ProfileAPI {
  constructor(private client: Client) {}

  async list(options?: RequestOptions): Promise<ProfileList> {
    const res = await this.client.GET('/profiles', options)
    if (res.error) {
      throw new APIError('unknown error')
    }

    const profileListResult = await profileListSchema.safeParseAsync(res.data)
    if (profileListResult.error) {
      throw new UnprocessableContentError(profileListResult.error.message)
    }

    return profileListResult.data
  }

  async get(id: UUID, options?: RequestOptions): Promise<Profile> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }

    const res = await this.client.GET('/profiles/{id}', {
      params: {
        path: {
          id: idResult.data,
        },
      },
      ...options,
    })
    if (res.error) {
      if (res.response.status === 404) {
        throw new NotFoundError(res.error.message)
      }

      throw new APIError(res.error.message)
    }

    const profileResult = await profileSchema.safeParseAsync(res.data)
    if (profileResult.error) {
      throw new UnprocessableContentError(profileResult.error.message)
    }

    return profileResult.data
  }

  async getActive(options?: RequestOptions): Promise<Profile> {
    const res = await this.client.GET('/profiles/@me', options)
    if (res.error) {
      throw new APIError('unknown error')
    }

    const profileResult = await profileSchema.safeParseAsync(res.data)
    if (profileResult.error) {
      throw new UnprocessableContentError(profileResult.error.message)
    }

    return profileResult.data
  }

  async create(
    body: ProfileCreate,
    options?: RequestOptions,
  ): Promise<Profile> {
    const bodyResult = await profileCreateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.POST('/profiles', {
      body: bodyResult.data,
      ...options,
    })
    if (res.error) {
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

  async update(
    id: UUID,
    body: ProfileUpdate,
    options?: RequestOptions,
  ): Promise<Profile> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }
    const bodyResult = await profileUpdateSchema.safeParseAsync(body)
    if (bodyResult.error) {
      throw new UnprocessableContentError(bodyResult.error.message)
    }

    const res = await this.client.PATCH('/profiles/{id}', {
      params: {
        path: {
          id: idResult.data,
        },
      },
      body: bodyResult.data,
      ...options,
    })
    if (res.error) {
      if (res.response.status === 404) {
        throw new NotFoundError(res.error.message)
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

  async delete(id: UUID, options?: RequestOptions): Promise<void> {
    const idResult = await uuidSchema.safeParseAsync(id)
    if (idResult.error) {
      throw new UnprocessableContentError(idResult.error.message)
    }

    const res = await this.client.DELETE('/profiles/{id}', {
      params: {
        path: {
          id: idResult.data,
        },
      },
      ...options,
    })
    if (res.error) {
      if (res.response.status === 404) {
        throw new NotFoundError(res.error.message)
      }
      if (res.response.status === 409) {
        throw new ConflictError(res.error.message)
      }

      throw new APIError(res.error.message)
    }
  }
}
