import { components, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type User = components['schemas']['User']

export interface AuthAPI {
  getActiveUser(): Promise<User>
}

export class HTTPAuthAPI implements AuthAPI {
  constructor(private client: Client<paths>) {}

  async getActiveUser(): Promise<User> {
    const res = await this.client.GET('/auth/@me')

    return res.data!
  }
}
