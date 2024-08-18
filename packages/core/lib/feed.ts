import type { z } from 'zod'
import {
  type ApiClient,
  Feed,
  FeedCreate,
  FeedList,
  FeedUpdate,
  File,
  get_ListFeeds,
} from './openapi.gen'

const ListFeedsQuery = get_ListFeeds.parameters.shape.query
export type ListFeedsQuery = z.infer<typeof ListFeedsQuery>

export interface FeedAPI {
  list(query: ListFeedsQuery): Promise<FeedList>

  get(id: string): Promise<Feed>

  create(data: FeedCreate): Promise<Feed>

  update(id: string, data: FeedUpdate): Promise<Feed>

  delete(id: string): Promise<void>

  import(data: File): Promise<void>
}

export class HTTPFeedAPI implements FeedAPI {
  constructor(private client: ApiClient) {}

  async list(query: ListFeedsQuery): Promise<FeedList> {
    return this.client
      .get('/feeds', {
        query: await ListFeedsQuery.parseAsync(query),
      })
      .then(FeedList.parseAsync)
  }

  async get(id: string): Promise<Feed> {
    return this.client
      .get('/feeds/{id}', {
        path: {
          id,
        },
      })
      .then(Feed.parseAsync)
  }

  async create(data: FeedCreate): Promise<Feed> {
    return this.client
      .post('/feeds', {
        body: await FeedCreate.parseAsync(data),
      })
      .then(Feed.parseAsync)
  }

  async update(id: string, data: FeedUpdate): Promise<Feed> {
    return this.client
      .patch('/feeds/{id}', {
        path: {
          id,
        },
        body: await FeedUpdate.parseAsync(data),
      })
      .then(Feed.parseAsync)
  }

  async delete(id: string): Promise<void> {
    return this.client
      .delete('/feeds/{id}', {
        path: {
          id,
        },
      })
      .then()
  }

  async import(data: File): Promise<void> {
    return this.client
      .post('/feeds/import', {
        body: await File.parseAsync(data),
      })
      .then()
  }
}
