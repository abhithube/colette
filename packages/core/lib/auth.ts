import { components, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type User = components['schemas']['User']
export type Register = components['schemas']['Register']
export type Login = components['schemas']['Login']

export interface AuthAPI {
  registerUser(data: Register): Promise<User>

  loginUser(data: Login): Promise<User>

  getActiveUser(): Promise<User>

  logoutUser(): Promise<void>
}

export class HTTPAuthAPI implements AuthAPI {
  constructor(private client: Client<paths>) {}

  async registerUser(data: Register): Promise<User> {
    const res = await this.client.POST('/auth/register', {
      body: data,
    })

    return res.data!
  }

  async loginUser(data: Login): Promise<User> {
    const res = await this.client.POST('/auth/login', {
      body: data,
    })

    return res.data!
  }

  async getActiveUser(): Promise<User> {
    const res = await this.client.GET('/auth/@me')

    return res.data!
  }

  async logoutUser(): Promise<void> {
    await this.client.POST('/auth/logout')
  }
}
