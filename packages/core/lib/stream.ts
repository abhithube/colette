import { FeedEntryList } from './feed-entry'
import {
  type ApiClient,
  Stream,
  StreamCreate,
  StreamUpdate,
  Paginated_Stream,
  get_ListStreamEntries,
} from './openapi.gen'
import { z } from 'zod'

export const StreamEntryListQuery = get_ListStreamEntries.parameters.shape.query
export type StreamEntryListQuery = z.infer<typeof StreamEntryListQuery>

export type StreamList = Paginated_Stream
export const StreamList = Paginated_Stream

export interface StreamAPI {
  list(): Promise<StreamList>

  get(id: string): Promise<Stream>

  create(data: StreamCreate): Promise<Stream>

  update(id: string, data: StreamUpdate): Promise<Stream>

  delete(id: string): Promise<void>

  listEntries(id: string, query: StreamEntryListQuery): Promise<FeedEntryList>
}

export class HTTPStreamAPI implements StreamAPI {
  constructor(private client: ApiClient) {}

  list(): Promise<StreamList> {
    return this.client.get('/streams').then(StreamList.parse)
  }

  get(id: string): Promise<Stream> {
    return this.client
      .get('/streams/{id}', {
        path: {
          id,
        },
      })
      .then(Stream.parse)
  }

  create(data: StreamCreate): Promise<Stream> {
    return this.client
      .post('/streams', {
        body: StreamCreate.parse(data),
      })
      .then(Stream.parse)
  }

  update(id: string, data: StreamUpdate): Promise<Stream> {
    return this.client
      .patch('/streams/{id}', {
        path: {
          id,
        },
        body: StreamUpdate.parse(data),
      })
      .then(Stream.parse)
  }

  delete(id: string): Promise<void> {
    return this.client
      .delete('/streams/{id}', {
        path: {
          id,
        },
      })
      .then()
  }

  listEntries(id: string, query: StreamEntryListQuery): Promise<FeedEntryList> {
    return this.client
      .get('/streams/{id}/entries', {
        path: {
          id,
        },
        query: StreamEntryListQuery.parse(query),
      })
      .then(FeedEntryList.parse)
  }
}
