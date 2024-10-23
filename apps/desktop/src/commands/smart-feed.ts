import {
  SmartFeed,
  type SmartFeedAPI,
  SmartFeedCreate,
  SmartFeedList,
  SmartFeedUpdate,
} from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class SmartFeedCommands implements SmartFeedAPI {
  list(): Promise<SmartFeedList> {
    return invoke('list_smart_feeds', {}).then(SmartFeedList.parse)
  }

  get(id: string): Promise<SmartFeed> {
    return invoke('get_smart_feed', { id }).then(SmartFeed.parse)
  }

  create(data: SmartFeedCreate): Promise<SmartFeed> {
    return invoke('create_smart_feed', {
      data: SmartFeedCreate.parse(data),
    }).then(SmartFeed.parse)
  }

  update(id: string, data: SmartFeedUpdate): Promise<SmartFeed> {
    return invoke('update_smart_feed', {
      id,
      data: SmartFeedUpdate.parse(data),
    }).then(SmartFeed.parse)
  }

  delete(id: string): Promise<void> {
    return invoke('delete_smart_feed', { id }).then()
  }
}
