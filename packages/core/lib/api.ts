import { type AuthAPI, HTTPAuthAPI } from './auth'
import { type BackupAPI, HTTPBackupAPI } from './backup'
import { type BookmarkAPI, HTTPBookmarkAPI } from './bookmark'
import { type CollectionAPI, HTTPCollectionAPI } from './collection'
import {
  APIError,
  BadGatewayError,
  ConflictError,
  NotFoundError,
  UnauthorizedError,
  UnprocessableContentError,
} from './error'
import { type FeedAPI, HTTPFeedAPI } from './feed'
import { type FeedEntryAPI, HTTPFeedEntryAPI } from './feed-entry'
import { BaseError, createApiClient } from './openapi.gen'
import { HTTPSmartFeedAPI, type SmartFeedAPI } from './smart-feed'
import { HTTPTagAPI, type TagAPI } from './tag'

export interface API {
  auth: AuthAPI
  backups: BackupAPI
  bookmarks: BookmarkAPI
  collections: CollectionAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  smartFeeds: SmartFeedAPI
  tags: TagAPI
}

export type HttpAPIOptions = Omit<RequestInit, 'method' | 'body'> & {
  baseUrl?: string
}

export class HttpAPI implements API {
  auth: AuthAPI
  backups: BackupAPI
  bookmarks: BookmarkAPI
  collections: CollectionAPI
  feedEntries: FeedEntryAPI
  feeds: FeedAPI
  smartFeeds: SmartFeedAPI
  tags: TagAPI

  constructor({ baseUrl, ...rest }: HttpAPIOptions) {
    const client = createApiClient((method, url, params) => {
      let finalUrl = url

      if (params?.path) {
        for (const [key, value] of Object.entries(params.path)) {
          finalUrl = finalUrl.replace(
            `{${key}}`,
            encodeURIComponent(value as any),
          )
        }
      }
      if (params?.query) {
        const search = new URLSearchParams()
        for (const [key, value] of Object.entries(params.query)) {
          if (value !== undefined) {
            search.append(key, encodeURIComponent(value as any))
          }
        }
        finalUrl = `${finalUrl}?${search.toString()}`
      }

      const headers: HeadersInit = {
        'Content-Type': 'application/json',
        ...rest.headers,
        ...params?.header,
      }
      const body: any = params?.body
        ? (headers as any)['Content-Type'] === 'application/json'
          ? JSON.stringify(params.body)
          : params.body
        : undefined

      return fetch(finalUrl, {
        method: method.toUpperCase(),
        body,
        headers,
        ...rest,
      }).then(async (res) => {
        if (res.status !== 204) {
          const data = await res.json()
          if (!res.ok) await handleError(data, res.status)

          return data
        }
      })
    }, baseUrl)

    this.auth = new HTTPAuthAPI(client)
    this.backups = new HTTPBackupAPI(client)
    this.bookmarks = new HTTPBookmarkAPI(client)
    this.collections = new HTTPCollectionAPI(client)
    this.feedEntries = new HTTPFeedEntryAPI(client)
    this.feeds = new HTTPFeedAPI(client)
    this.smartFeeds = new HTTPSmartFeedAPI(client)
    this.tags = new HTTPTagAPI(client)
  }
}

async function handleError(data: unknown, status: number) {
  const parsed = BaseError.safeParse(data)
  if (parsed.error) {
    throw new UnprocessableContentError(parsed.error.message)
  }

  const message = parsed.data.message
  switch (status) {
    case 401:
      throw new UnauthorizedError(message)
    case 404:
      throw new NotFoundError(message)
    case 409:
      throw new ConflictError(message)
    case 422:
      throw new UnprocessableContentError(message)
    case 502:
      throw new BadGatewayError(message)
    default:
      throw new APIError(message)
  }
}
