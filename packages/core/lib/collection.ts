import {
  type ApiClient,
  Collection,
  CollectionCreate,
  CollectionUpdate,
  get_ListCollections,
  Paginated_Collection,
} from './openapi.gen'
import { z } from 'zod'

export const CollectionListQuery = get_ListCollections.parameters.shape.query
export type CollectionListQuery = z.infer<typeof CollectionListQuery>

export type CollectionList = Paginated_Collection
export const CollectionList = Paginated_Collection

export interface CollectionAPI {
  list(query: CollectionListQuery): Promise<CollectionList>

  get(id: string): Promise<Collection>

  create(data: CollectionCreate): Promise<Collection>

  update(id: string, data: CollectionUpdate): Promise<Collection>

  delete(id: string): Promise<void>
}

export class HTTPCollectionAPI implements CollectionAPI {
  constructor(private client: ApiClient) {}

  list(query: CollectionListQuery): Promise<CollectionList> {
    return this.client
      .get('/collections', {
        query: CollectionListQuery.parse(query),
      })
      .then(CollectionList.parse)
  }

  get(id: string): Promise<Collection> {
    return this.client
      .get('/collections/{id}', {
        path: {
          id,
        },
      })
      .then(Collection.parse)
  }

  create(data: CollectionCreate): Promise<Collection> {
    return this.client
      .post('/collections', {
        body: CollectionCreate.parse(data),
      })
      .then(Collection.parse)
  }

  update(id: string, data: CollectionUpdate): Promise<Collection> {
    return this.client
      .patch('/collections/{id}', {
        path: {
          id,
        },
        body: CollectionUpdate.parse(data),
      })
      .then(Collection.parse)
  }

  delete(id: string): Promise<void> {
    return this.client
      .delete('/collections/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
