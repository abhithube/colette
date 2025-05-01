import { components, operations, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type Tag = components['schemas']['Tag']
export type TagDetails = components['schemas']['TagDetails']
export type TagCreate = components['schemas']['TagCreate']
export type TagUpdate = components['schemas']['TagUpdate']
export type TagDetailsList = components['schemas']['Paginated_TagDetails']

export type TagListQuery = NonNullable<
  operations['listTags']['parameters']['query']
>
export type TagGetQuery = NonNullable<
  operations['getTag']['parameters']['query']
>

export interface TagAPI {
  listTags(query: TagListQuery): Promise<TagDetailsList>

  getTag(id: string, query: TagGetQuery): Promise<TagDetails>

  createTag(data: TagCreate): Promise<Tag>

  updateTag(id: string, data: TagUpdate): Promise<Tag>

  deleteTag(id: string): Promise<void>
}

export class HTTPTagAPI implements TagAPI {
  constructor(private client: Client<paths>) {}

  async listTags(query: TagListQuery): Promise<TagDetailsList> {
    const res = await this.client.GET('/tags', {
      params: {
        query,
      },
    })

    return res.data!
  }

  async getTag(id: string, query: TagGetQuery): Promise<TagDetails> {
    const res = await this.client.GET('/tags/{id}', {
      params: {
        path: {
          id,
        },
        query,
      },
    })

    return res.data!
  }

  async createTag(data: TagCreate): Promise<Tag> {
    const res = await this.client.POST('/tags', {
      body: data,
    })

    return res.data!
  }

  async updateTag(id: string, data: TagUpdate): Promise<Tag> {
    const res = await this.client.PATCH('/tags/{id}', {
      params: {
        path: {
          id,
        },
      },
      body: data,
    })

    return res.data!
  }

  async deleteTag(id: string): Promise<void> {
    await this.client.DELETE('/tags/{id}', {
      params: {
        path: {
          id,
        },
      },
    })
  }
}
