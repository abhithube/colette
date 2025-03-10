import {
  type ApiClient,
  Stream,
  StreamCreate,
  StreamUpdate,
  Paginated_Stream,
} from './openapi.gen'

export type StreamList = Paginated_Stream
export const StreamList = Paginated_Stream

export interface StreamAPI {
  listStreams(): Promise<StreamList>

  getStream(id: string): Promise<Stream>

  createStream(data: StreamCreate): Promise<Stream>

  updateStream(id: string, data: StreamUpdate): Promise<Stream>

  deleteStream(id: string): Promise<void>
}

export class HTTPStreamAPI implements StreamAPI {
  constructor(private client: ApiClient) {}

  listStreams(): Promise<StreamList> {
    return this.client.get('/streams').then(StreamList.parse)
  }

  getStream(id: string): Promise<Stream> {
    return this.client
      .get('/streams/{id}', {
        path: {
          id,
        },
      })
      .then(Stream.parse)
  }

  createStream(data: StreamCreate): Promise<Stream> {
    return this.client
      .post('/streams', {
        body: StreamCreate.parse(data),
      })
      .then(Stream.parse)
  }

  updateStream(id: string, data: StreamUpdate): Promise<Stream> {
    return this.client
      .patch('/streams/{id}', {
        path: {
          id,
        },
        body: StreamUpdate.parse(data),
      })
      .then(Stream.parse)
  }

  deleteStream(id: string): Promise<void> {
    return this.client
      .delete('/streams/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
