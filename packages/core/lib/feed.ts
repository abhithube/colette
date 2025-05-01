import { components, paths } from './openapi'
import { Client } from 'openapi-fetch'

export type Feed = components['schemas']['Feed']
export type FeedDetect = components['schemas']['FeedDetect']
export type FeedDetected = components['schemas']['FeedDetected']
export type FeedScrape = components['schemas']['FeedScrape']

export interface FeedAPI {
  detectFeeds(data: FeedDetect): Promise<FeedDetected[]>

  scrapeFeed(data: FeedScrape): Promise<Feed>
}

export class HTTPFeedAPI implements FeedAPI {
  constructor(private client: Client<paths>) {}

  async detectFeeds(data: FeedDetect): Promise<FeedDetected[]> {
    const res = await this.client.POST('/feeds/detect', {
      body: data,
    })

    return res.data!
  }

  async scrapeFeed(data: FeedScrape): Promise<Feed> {
    const res = await this.client.POST('/feeds/scrape', {
      body: data,
    })

    return res.data!
  }
}
