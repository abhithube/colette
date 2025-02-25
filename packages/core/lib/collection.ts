import { BookmarkList } from './bookmark'
import {
  type ApiClient,
  Collection,
  CollectionCreate,
  CollectionUpdate,
  get_ListCollectionBookmarks,
  Paginated_Collection,
} from './openapi.gen'
import { z } from 'zod'

export const CollectionBookmarkListQuery =
  get_ListCollectionBookmarks.parameters.shape.query
export type CollectionBookmarkListQuery = z.infer<
  typeof CollectionBookmarkListQuery
>

export type CollectionList = Paginated_Collection
export const CollectionList = Paginated_Collection

export interface CollectionAPI {
  list(): Promise<CollectionList>

  get(id: string): Promise<Collection>

  create(data: CollectionCreate): Promise<Collection>

  update(id: string, data: CollectionUpdate): Promise<Collection>

  delete(id: string): Promise<void>

  listBookmarks(
    id: string,
    query: CollectionBookmarkListQuery,
  ): Promise<BookmarkList>
}

export class HTTPCollectionAPI implements CollectionAPI {
  constructor(private client: ApiClient) {}

  list(): Promise<CollectionList> {
    return this.client.get('/collections').then(CollectionList.parse)
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

  listBookmarks(
    id: string,
    query: CollectionBookmarkListQuery,
  ): Promise<BookmarkList> {
    return this.client
      .get('/collections/{id}/bookmarks', {
        path: {
          id,
        },
        query: CollectionBookmarkListQuery.parse(query),
      })
      .then(BookmarkList.parse)
  }
}
