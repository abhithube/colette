import { components, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type Stream = components['schemas']['Stream']
export type StreamCreate = components['schemas']['StreamCreate']
export type StreamUpdate = components['schemas']['StreamUpdate']
export type StreamList = components['schemas']['Paginated_Stream']

export interface StreamAPI {
  listStreams(): Promise<StreamList>

  getStream(id: string): Promise<Stream>

  createStream(data: StreamCreate): Promise<Stream>

  updateStream(id: string, data: StreamUpdate): Promise<Stream>

  deleteStream(id: string): Promise<void>
}

export class HTTPStreamAPI implements StreamAPI {
  constructor(private client: Client<paths>) {}

  async listStreams(): Promise<StreamList> {
    const res = await this.client.GET('/streams')

    return res.data!
  }

  async getStream(id: string): Promise<Stream> {
    const res = await this.client.GET('/streams/{id}', {
      params: {
        path: {
          id,
        },
      },
    })

    return res.data!
  }

  async createStream(data: StreamCreate): Promise<Stream> {
    const res = await this.client.POST('/streams', {
      body: data,
    })

    return res.data!
  }

  async updateStream(id: string, data: StreamUpdate): Promise<Stream> {
    const res = await this.client.PATCH('/streams/{id}', {
      params: {
        path: {
          id,
        },
      },
      body: data,
    })

    return res.data!
  }

  async deleteStream(id: string): Promise<void> {
    await this.client.DELETE('/streams/{id}', {
      params: {
        path: {
          id,
        },
      },
    })
  }
}
