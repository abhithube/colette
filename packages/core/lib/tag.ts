import type { z } from 'zod'
import {
  type ApiClient,
  Tag,
  TagCreate,
  TagList,
  TagUpdate,
  get_ListTags,
} from './openapi.gen'

const ListTagsQuery = get_ListTags.parameters.shape.query
export type ListTagsQuery = z.infer<typeof ListTagsQuery>

export interface TagAPI {
  list(query: ListTagsQuery): Promise<TagList>

  get(id: string): Promise<Tag>

  create(body: TagCreate): Promise<Tag>

  update(id: string, body: TagUpdate): Promise<Tag>

  delete(id: string): Promise<void>
}

export class HTTPTagAPI implements TagAPI {
  constructor(private client: ApiClient) {}

  async list(query: ListTagsQuery): Promise<TagList> {
    return this.client
      .get('/tags', {
        query: await ListTagsQuery.parseAsync(query),
      })
      .then(TagList.parseAsync)
  }

  async get(id: string): Promise<Tag> {
    return this.client
      .get('/tags/{id}', {
        path: {
          id,
        },
      })
      .then(Tag.parseAsync)
  }

  async create(body: TagCreate): Promise<Tag> {
    return this.client
      .post('/tags', {
        body: await TagCreate.parseAsync(body),
      })
      .then(Tag.parseAsync)
  }

  async update(id: string, body: TagUpdate): Promise<Tag> {
    return this.client
      .patch('/tags/{id}', {
        path: {
          id,
        },
        body: await TagUpdate.parseAsync(body),
      })
      .then(Tag.parseAsync)
  }

  async delete(id: string): Promise<void> {
    return this.client
      .delete('/tags/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
