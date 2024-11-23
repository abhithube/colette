import {
  Feed,
  type FeedAPI,
  FeedCreate,
  FeedDetect,
  FeedDetectedList,
  FeedList,
  FeedListQuery,
  FeedUpdate,
} from '@colette/core'
import { invoke } from '@tauri-apps/api/core'

export class FeedCommands implements FeedAPI {
  list(query: FeedListQuery): Promise<FeedList> {
    return invoke('list_feeds', { query: FeedListQuery.parse(query) }).then(
      FeedList.parse,
    )
  }

  get(id: string): Promise<Feed> {
    return invoke('get_feed', { id }).then(Feed.parse)
  }

  create(data: FeedCreate): Promise<Feed> {
    return invoke('create_feed', { data: FeedCreate.parse(data) }).then(
      Feed.parse,
    )
  }

  update(id: string, data: FeedUpdate): Promise<Feed> {
    return invoke('update_feed', {
      id,
      data: FeedUpdate.parse(data),
    }).then(Feed.parse)
  }

  delete(id: string): Promise<void> {
    return invoke('delete_feed', { id }).then()
  }

  detect(data: FeedDetect): Promise<FeedDetectedList> {
    return invoke('detect_feeds', { data: FeedDetect.parse(data) }).then(
      FeedDetectedList.parse,
    )
  }
}
