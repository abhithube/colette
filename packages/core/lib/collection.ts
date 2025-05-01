import { components, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type Collection = components['schemas']['Collection']
export type CollectionCreate = components['schemas']['CollectionCreate']
export type CollectionUpdate = components['schemas']['CollectionUpdate']
export type CollectionList = components['schemas']['Paginated_Collection']

export interface CollectionAPI {
  listCollections(): Promise<CollectionList>

  getCollection(id: string): Promise<Collection>

  createCollection(data: CollectionCreate): Promise<Collection>

  updateCollection(id: string, data: CollectionUpdate): Promise<Collection>

  deleteCollection(id: string): Promise<void>
}

export class HTTPCollectionAPI implements CollectionAPI {
  constructor(private client: Client<paths>) {}

  async listCollections(): Promise<CollectionList> {
    const res = await this.client.GET('/collections')

    return res.data!
  }

  async getCollection(id: string): Promise<Collection> {
    const res = await this.client.GET('/collections/{id}', {
      params: {
        path: {
          id,
        },
      },
    })

    return res.data!
  }

  async createCollection(data: CollectionCreate): Promise<Collection> {
    const res = await this.client.POST('/collections', {
      body: data,
    })

    return res.data!
  }

  async updateCollection(
    id: string,
    data: CollectionUpdate,
  ): Promise<Collection> {
    const res = await this.client.PATCH('/collections/{id}', {
      params: {
        path: {
          id,
        },
      },
      body: data,
    })

    return res.data!
  }

  async deleteCollection(id: string): Promise<void> {
    await this.client.DELETE('/collections/{id}', {
      params: {
        path: {
          id,
        },
      },
    })
  }
}
