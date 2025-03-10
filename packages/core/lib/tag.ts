import {
  type ApiClient,
  Paginated_Tag,
  Tag,
  TagCreate,
  TagUpdate,
  get_ListTags,
} from './openapi.gen'
import type { z } from 'zod'

export const TagListQuery = get_ListTags.parameters.shape.query
export type TagListQuery = z.infer<typeof TagListQuery>

export type TagList = Paginated_Tag
export const TagList = Paginated_Tag

export interface TagAPI {
  listTags(query: TagListQuery): Promise<TagList>

  getTag(id: string): Promise<Tag>

  createTag(data: TagCreate): Promise<Tag>

  updateTag(id: string, data: TagUpdate): Promise<Tag>

  deleteTag(id: string): Promise<void>
}

export class HTTPTagAPI implements TagAPI {
  constructor(private client: ApiClient) {}

  listTags(query: TagListQuery): Promise<TagList> {
    return this.client
      .get('/tags', {
        query: TagListQuery.parse(query),
      })
      .then(TagList.parse)
  }

  getTag(id: string): Promise<Tag> {
    return this.client
      .get('/tags/{id}', {
        path: {
          id,
        },
      })
      .then(Tag.parse)
  }

  createTag(data: TagCreate): Promise<Tag> {
    return this.client
      .post('/tags', {
        body: TagCreate.parse(data),
      })
      .then(Tag.parse)
  }

  updateTag(id: string, data: TagUpdate): Promise<Tag> {
    return this.client
      .patch('/tags/{id}', {
        path: {
          id,
        },
        body: TagUpdate.parse(data),
      })
      .then(Tag.parse)
  }

  deleteTag(id: string): Promise<void> {
    return this.client
      .delete('/tags/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
