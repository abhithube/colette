import type { z } from 'zod'
import {
  type ApiClient,
  Feed,
  FeedCreate,
  FeedDetect,
  FeedUpdate,
  Paginated_Feed,
  Paginated_FeedDetected,
  get_ListFeeds,
} from './openapi.gen'

export const FeedListQuery = get_ListFeeds.parameters.shape.query
export type FeedListQuery = z.infer<typeof FeedListQuery>

export type FeedList = Paginated_Feed
export const FeedList = Paginated_Feed
export const FeedDetectedList = Paginated_FeedDetected
export type FeedDetectedList = Paginated_FeedDetected

export interface FeedAPI {
  list(query: FeedListQuery): Promise<FeedList>

  get(id: string): Promise<Feed>

  create(data: FeedCreate): Promise<Feed>
  
  update(id: string, data: FeedUpdate): Promise<Feed>
  
  delete(id: string): Promise<void>

  detect(data: FeedDetect): Promise<FeedDetectedList>
}

export class HTTPFeedAPI implements FeedAPI {
  constructor(private client: ApiClient) {}

  list(query: FeedListQuery): Promise<FeedList> {
    return this.client
      .get('/feeds', {
        query: FeedListQuery.parse(query),
      })
      .then(FeedList.parse)
  }

  get(id: string): Promise<Feed> {
    return this.client
      .get('/feeds/{id}', {
        path: {
          id,
        },
      })
      .then(Feed.parse)
  }

  create(data: FeedCreate): Promise<Feed> {
    return this.client
      .post('/feeds', {
        body: FeedCreate.parse(data),
      })
      .then(Feed.parse)
  }

  update(id: string, data: FeedUpdate): Promise<Feed> {
    return this.client
      .patch('/feeds/{id}', {
        path: {
          id,
        },
        body: FeedUpdate.parse(data),
      })
      .then(Feed.parse)
  }

  delete(id: string): Promise<void> {
    return this.client
      .delete('/feeds/{id}', {
        path: {
          id,
        },
      })
      .then()
  }

  detect(data: FeedDetect): Promise<FeedDetectedList> {
    return this.client
      .post('/feeds/detect', {
        body: FeedDetect.parse(data),
      })
      .then(FeedDetectedList.parse)
  }
}
