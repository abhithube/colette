import {
  Collection,
  type CollectionAPI,
  CollectionCreate,
  CollectionList,
  CollectionUpdate,
} from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class CollectionCommands implements CollectionAPI {
  list(): Promise<CollectionList> {
    return invoke('list_collections').then(CollectionList.parse)
  }

  get(id: string): Promise<Collection> {
    return invoke('get_collection', { id }).then(Collection.parse)
  }

  create(data: CollectionCreate): Promise<Collection> {
    return invoke('create_collection', {
      data: CollectionCreate.parse(data),
    }).then(Collection.parse)
  }

  update(id: string, data: CollectionUpdate): Promise<Collection> {
    return invoke('update_collection', {
      id,
      data: CollectionUpdate.parse(data),
    }).then(Collection.parse)
  }

  delete(id: string): Promise<void> {
    return invoke('delete_collection', { id }).then()
  }
}
