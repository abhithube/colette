import { FeedFolderItem } from './feed-folder-item'
import { FeedItem } from './feed-item'
import { listFeedTreeItemsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'

export const FeedFolderContents: FC<{ folderId?: string }> = ({ folderId }) => {
  const api = useAPI()

  const { data: items, isLoading } = useQuery(
    listFeedTreeItemsOptions(api, { folderId }),
  )

  if (isLoading || !items) return

  return items.data.map((item) =>
    item.type === 'feed' ? (
      <FeedItem key={item.data.id} feed={item.data} />
    ) : (
      <FeedFolderItem key={item.data.id} folder={item.data} />
    ),
  )
}
