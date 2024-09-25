import {
  Tag,
  type TagAPI,
  TagCreate,
  TagList,
  TagListQuery,
  TagUpdate,
} from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class TagCommands implements TagAPI {
  list(query: TagListQuery): Promise<TagList> {
    return invoke('list_tags', {
      query: TagListQuery.parse(query),
    }).then(TagList.parse)
  }

  get(id: string): Promise<Tag> {
    return invoke('get_tag', { id }).then(Tag.parse)
  }

  create(data: TagCreate): Promise<Tag> {
    return invoke('create_tag', { data: TagCreate.parse(data) }).then(Tag.parse)
  }

  update(id: string, data: TagUpdate): Promise<Tag> {
    return invoke('update_tag', {
      id,
      data: TagUpdate.parse(data),
    }).then(Tag.parse)
  }

  delete(id: string): Promise<void> {
    return invoke('delete_tag', { id }).then()
  }
}
