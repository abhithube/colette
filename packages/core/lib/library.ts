import {
  type ApiClient,
  Paginated_FeedTreeItem,
  get_ListFeedTree,
  Paginated_CollectionTreeItem,
} from './openapi.gen'
import { z } from 'zod'

export const TreeListQuery = get_ListFeedTree.parameters.shape.query
export type TreeListQuery = z.infer<typeof TreeListQuery>

export type FeedTreeItemList = Paginated_FeedTreeItem
export const FeedTreeItemList = Paginated_FeedTreeItem

export type CollectionTreeItemList = Paginated_CollectionTreeItem
export const CollectionTreeItemList = Paginated_CollectionTreeItem

export interface LibraryAPI {
  listFeedTree(query: TreeListQuery): Promise<FeedTreeItemList>

  listCollectionTree(query: TreeListQuery): Promise<CollectionTreeItemList>
}

export class HTTPLibraryAPI implements LibraryAPI {
  constructor(private client: ApiClient) {}

  listFeedTree(query: TreeListQuery): Promise<FeedTreeItemList> {
    return this.client
      .get('/library/feedTree', {
        query: TreeListQuery.parse(query),
      })
      .then(FeedTreeItemList.parse)
  }

  listCollectionTree(query: TreeListQuery): Promise<CollectionTreeItemList> {
    return this.client
      .get('/library/collectionTree', {
        query: TreeListQuery.parse(query),
      })
      .then(CollectionTreeItemList.parse)
  }
}
