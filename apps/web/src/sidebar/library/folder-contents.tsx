import { CollectionItem } from './collections/collection-item'
import { FeedItem } from './feeds/feed-item'
import { FolderItem } from './folder-item'
import { listLibraryItemsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'

export const FolderContents: FC<{ folderId?: string }> = ({ folderId }) => {
  const api = useAPI()

  const { data: items, isLoading } = useQuery(
    listLibraryItemsOptions(api, { folderId }),
  )

  if (isLoading || !items) return

  return items.data.map((item) =>
    item.type === 'feed' ? (
      <FeedItem key={item.data.id} feed={item.data} />
    ) : item.type === 'collection' ? (
      <CollectionItem key={item.data.id} collection={item.data} />
    ) : (
      <FolderItem key={item.data.id} folder={item.data} />
    ),
  )
}
