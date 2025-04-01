import {
  type ApiClient,
  Paginated_TagDetails,
  Tag,
  TagCreate,
  TagDetails,
  TagUpdate,
  get_GetTag,
  get_ListTags,
} from './openapi.gen'
import type { z } from 'zod'

export const TagListQuery = get_ListTags.parameters.shape.query
export type TagListQuery = z.infer<typeof TagListQuery>

export const TagGetQuery = get_GetTag.parameters.shape.query
export type TagGetQuery = z.infer<typeof TagGetQuery>

export type TagDetailsList = Paginated_TagDetails
export const TagDetailsList = Paginated_TagDetails

export interface TagAPI {
  listTags(query: TagListQuery): Promise<TagDetailsList>

  getTag(id: string, query: TagGetQuery): Promise<TagDetails>

  createTag(data: TagCreate): Promise<Tag>

  updateTag(id: string, data: TagUpdate): Promise<Tag>

  deleteTag(id: string): Promise<void>
}

export class HTTPTagAPI implements TagAPI {
  constructor(private client: ApiClient) {}

  listTags(query: TagListQuery): Promise<TagDetailsList> {
    return this.client
      .get('/tags', {
        query: TagListQuery.parse(query),
      })
      .then(TagDetailsList.parse)
  }

  getTag(id: string, query: TagGetQuery): Promise<TagDetails> {
    return this.client
      .get('/tags/{id}', {
        path: {
          id,
        },
        query: TagGetQuery.parse(query),
      })
      .then(TagDetails.parse)
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
