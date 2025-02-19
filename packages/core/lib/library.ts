import {
  type ApiClient,
  Paginated_LibraryItem,
  get_ListLibraryItems,
} from './openapi.gen'
import { z } from 'zod'

export const LibraryItemListQuery = get_ListLibraryItems.parameters.shape.query
export type LibraryItemListQuery = z.infer<typeof LibraryItemListQuery>

export type LibraryItemList = Paginated_LibraryItem
export const LibraryItemList = Paginated_LibraryItem

export interface LibraryAPI {
  list(query: LibraryItemListQuery): Promise<LibraryItemList>
}

export class HTTPLibraryAPI implements LibraryAPI {
  constructor(private client: ApiClient) {}

  list(query: LibraryItemListQuery): Promise<LibraryItemList> {
    return this.client
      .get('/library', {
        query: LibraryItemListQuery.parse(query),
      })
      .then(LibraryItemList.parse)
  }
}
