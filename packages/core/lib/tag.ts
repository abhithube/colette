import type { z } from 'zod'
import {
  type ApiClient,
  Tag,
  TagCreate,
  TagList,
  TagUpdate,
  get_ListTags,
} from './openapi.gen'

export const TagListQuery = get_ListTags.parameters.shape.query
export type TagListQuery = z.infer<typeof TagListQuery>

export interface TagAPI {
  list(query: TagListQuery): Promise<TagList>

  get(id: string): Promise<Tag>

  create(data: TagCreate): Promise<Tag>

  update(id: string, data: TagUpdate): Promise<Tag>

  delete(id: string): Promise<void>
}

export class HTTPTagAPI implements TagAPI {
  constructor(private client: ApiClient) {}

  list(query: TagListQuery): Promise<TagList> {
    return this.client
      .get('/tags', {
        query: TagListQuery.parse(query),
      })
      .then(TagList.parse)
  }

  get(id: string): Promise<Tag> {
    return this.client
      .get('/tags/{id}', {
        path: {
          id,
        },
      })
      .then(Tag.parse)
  }

  create(body: TagCreate): Promise<Tag> {
    return this.client
      .post('/tags', {
        body: TagCreate.parse(body),
      })
      .then(Tag.parse)
  }

  update(id: string, body: TagUpdate): Promise<Tag> {
    return this.client
      .patch('/tags/{id}', {
        path: {
          id,
        },
        body: TagUpdate.parse(body),
      })
      .then(Tag.parse)
  }

  delete(id: string): Promise<void> {
    return this.client
      .delete('/tags/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
