import {
  type ApiClient,
  Folder,
  FolderCreate,
  FolderUpdate,
  get_ListFolders,
  Paginated_Folder,
} from './openapi.gen'
import { z } from 'zod'

export const FolderListQuery = get_ListFolders.parameters.shape.query
export type FolderListQuery = z.infer<typeof FolderListQuery>

export type FolderList = Paginated_Folder
export const FolderList = Paginated_Folder

export interface FolderAPI {
  list(query: FolderListQuery): Promise<FolderList>

  get(id: string): Promise<Folder>

  create(data: FolderCreate): Promise<Folder>

  update(id: string, data: FolderUpdate): Promise<Folder>

  delete(id: string): Promise<void>
}

export class HTTPFolderAPI implements FolderAPI {
  constructor(private client: ApiClient) {}

  list(query: FolderListQuery): Promise<FolderList> {
    return this.client
      .get('/folders', {
        query: FolderListQuery.parse(query),
      })
      .then(FolderList.parse)
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

  create(data: FolderCreate): Promise<Folder> {
    return this.client
      .post('/folders', {
        body: FolderCreate.parse(data),
      })
      .then(Folder.parse)
  }

  update(id: string, data: FolderUpdate): Promise<Folder> {
    return this.client
      .patch('/folders/{id}', {
        path: {
          id,
        },
        body: FolderUpdate.parse(data),
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
