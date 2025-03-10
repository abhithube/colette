import {
  type ApiClient,
  Collection,
  CollectionCreate,
  CollectionUpdate,
  Paginated_Collection,
} from './openapi.gen'

export type CollectionList = Paginated_Collection
export const CollectionList = Paginated_Collection

export interface CollectionAPI {
  listCollections(): Promise<CollectionList>

  getCollection(id: string): Promise<Collection>

  createCollection(data: CollectionCreate): Promise<Collection>

  updateCollection(id: string, data: CollectionUpdate): Promise<Collection>

  deleteCollection(id: string): Promise<void>
}

export class HTTPCollectionAPI implements CollectionAPI {
  constructor(private client: ApiClient) {}

  listCollections(): Promise<CollectionList> {
    return this.client.get('/collections').then(CollectionList.parse)
  }

  getCollection(id: string): Promise<Collection> {
    return this.client
      .get('/collections/{id}', {
        path: {
          id,
        },
      })
      .then(Collection.parse)
  }

  createCollection(data: CollectionCreate): Promise<Collection> {
    return this.client
      .post('/collections', {
        body: CollectionCreate.parse(data),
      })
      .then(Collection.parse)
  }

  updateCollection(id: string, data: CollectionUpdate): Promise<Collection> {
    return this.client
      .patch('/collections/{id}', {
        path: {
          id,
        },
        body: CollectionUpdate.parse(data),
      })
      .then(Collection.parse)
  }

  deleteCollection(id: string): Promise<void> {
    return this.client
      .delete('/collections/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
