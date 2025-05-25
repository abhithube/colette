import { components, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type Config = components['schemas']['Config']

export interface ConfigAPI {
  getConfig(): Promise<Config>
}

export class HTTPConfigAPI implements ConfigAPI {
  constructor(private client: Client<paths>) {}

  async getConfig(): Promise<Config> {
    const res = await this.client.GET('/config')

    return res.data!
  }
}
