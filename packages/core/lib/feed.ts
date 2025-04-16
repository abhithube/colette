import {
  type ApiClient,
  Feed,
  FeedDetect,
  FeedDetected,
  FeedScrape,
} from './openapi.gen'
import { z } from 'zod'

export interface FeedAPI {
  detectFeeds(data: FeedDetect): Promise<FeedDetected[]>

  scrapeFeed(data: FeedScrape): Promise<Feed>
}

export class HTTPFeedAPI implements FeedAPI {
  constructor(private client: ApiClient) {}

  detectFeeds(data: FeedDetect): Promise<FeedDetected[]> {
    return this.client
      .post('/feeds/detect', {
        body: FeedDetect.parse(data),
      })
      .then(z.array(FeedDetected).parse)
  }

  scrapeFeed(data: FeedScrape): Promise<Feed> {
    return this.client
      .post('/feeds/scrape', {
        body: FeedScrape.parse(data),
      })
      .then(Feed.parse)
  }
}
