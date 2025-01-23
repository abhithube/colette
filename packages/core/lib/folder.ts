import {
  type ApiClient,
  Folder,
  FolderCreate,
  FolderUpdate,
  Paginated_Folder,
} from './openapi.gen'

export type FolderList = Paginated_Folder
export const FolderList = Paginated_Folder

export interface FolderAPI {
  list(): Promise<FolderList>

  get(id: string): Promise<Folder>

  create(data: FolderCreate): Promise<Folder>

  update(id: string, data: FolderUpdate): Promise<Folder>

  delete(id: string): Promise<void>
}

export class HTTPFolderAPI implements FolderAPI {
  constructor(private client: ApiClient) {}

  list(): Promise<FolderList> {
    return this.client.get('/folders').then(FolderList.parse)
  }

  get(id: string): Promise<Folder> {
    return this.client
      .get('/folders/{id}', {
        path: {
          id,
        },
      })
      .then(Folder.parse)
  }

  create(body: FolderCreate): Promise<Folder> {
    return this.client
      .post('/folders', {
        body: FolderCreate.parse(body),
      })
      .then(Folder.parse)
  }

  update(id: string, body: FolderUpdate): Promise<Folder> {
    return this.client
      .patch('/folders/{id}', {
        path: {
          id,
        },
        body: FolderUpdate.parse(body),
      })
      .then(Folder.parse)
  }

  delete(id: string): Promise<void> {
    return this.client
      .delete('/folders/{id}', {
        path: {
          id,
        },
      })
      .then()
  }
}
