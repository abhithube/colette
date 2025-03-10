import { type ApiClient, DetectedResponse, FeedDetect } from './openapi.gen'

export interface FeedAPI {
  detectFeeds(data: FeedDetect): Promise<DetectedResponse>
}

export class HTTPFeedAPI implements FeedAPI {
  constructor(private client: ApiClient) {}

  detectFeeds(data: FeedDetect): Promise<DetectedResponse> {
    return this.client
      .post('/feeds/detect', {
        body: FeedDetect.parse(data),
      })
      .then(DetectedResponse.parse)
  }
}
