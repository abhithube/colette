import { type AuthAPI, HTTPAuthAPI } from './auth'
import { type BookmarkAPI, HTTPBookmarkAPI } from './bookmark'
import { type CollectionAPI, HTTPCollectionAPI } from './collection'
import { type ConfigAPI, HTTPConfigAPI } from './config'
import {
  BadGatewayError,
  ConflictError,
  ForbiddenError,
  NotFoundError,
  ServerError,
  UnauthorizedError,
  UnprocessableContentError,
} from './error'
import { type FeedAPI, HTTPFeedAPI } from './feed'
import { type FeedEntryAPI, HTTPFeedEntryAPI } from './feed-entry'
import { components, paths } from './openapi'
import { HTTPStreamAPI, type StreamAPI } from './stream'
import { HTTPSubscriptionAPI, type SubscriptionAPI } from './subscription'
import {
  HTTPSubscriptionEntryAPI,
  type SubscriptionEntryAPI,
} from './subscription-entry'
import { HTTPTagAPI, type TagAPI } from './tag'
import createClient, { Middleware } from 'openapi-fetch'

export type ApiError = components['schemas']['ApiError']
export type ApiErrorCode = components['schemas']['ApiErrorCode']

export interface API {
  auth: AuthAPI
  bookmarks: BookmarkAPI
  collections: CollectionAPI
  config: ConfigAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  streams: StreamAPI
  subscriptionEntries: SubscriptionEntryAPI
  subscriptions: SubscriptionAPI
  tags: TagAPI
}

const authMiddleware: Middleware = {
  onRequest: async ({ request }) => {
    const accessToken = localStorage.getItem('colette-access-token')
    if (accessToken) {
      request.headers.set('Authorization', `Bearer ${accessToken}`)
    }

    return request
  },
}

const errorMiddleware: Middleware = {
  async onResponse({ response }) {
    if (!response.ok) {
      const err = (await response.json()) as ApiError

      switch (response.status) {
        case 401:
          throw new UnauthorizedError(err.message)
        case 403:
          throw new ForbiddenError(err.message)
        case 404:
          throw new NotFoundError(err.message)
        case 409:
          throw new ConflictError(err.message)
        case 422:
          throw new UnprocessableContentError(err.message)
        case 502:
          throw new BadGatewayError(err.message)
        default:
          throw new ServerError(err.message)
      }
    }
  },
}

export type HttpAPIOptions = Omit<RequestInit, 'method' | 'body'> & {
  baseUrl?: string
}

export class HttpAPI implements API {
  auth: AuthAPI
  bookmarks: BookmarkAPI
  collections: CollectionAPI
  config: ConfigAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  streams: StreamAPI
  subscriptionEntries: SubscriptionEntryAPI
  subscriptions: SubscriptionAPI
  tags: TagAPI

  constructor({ baseUrl, ...rest }: HttpAPIOptions) {
    const client = createClient<paths>({ baseUrl, ...rest })
    client.use(authMiddleware, errorMiddleware)

    this.auth = new HTTPAuthAPI(client)
    this.bookmarks = new HTTPBookmarkAPI(client)
    this.collections = new HTTPCollectionAPI(client)
    this.config = new HTTPConfigAPI(client)
    this.feedEntries = new HTTPFeedEntryAPI(client)
    this.feeds = new HTTPFeedAPI(client)
    this.streams = new HTTPStreamAPI(client)
    this.subscriptionEntries = new HTTPSubscriptionEntryAPI(client)
    this.subscriptions = new HTTPSubscriptionAPI(client)
    this.tags = new HTTPTagAPI(client)
  }
}
