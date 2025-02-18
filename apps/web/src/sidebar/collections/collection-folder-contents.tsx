import { CollectionFolderItem } from './collection-folder-item'
import { CollectionItem } from './collection-item'
import { listCollectionTreeItemsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'

export const CollectionFolderContents: FC<{ folderId?: string }> = ({
  folderId,
}) => {
  const api = useAPI()

  const { data: items, isLoading } = useQuery(
    listCollectionTreeItemsOptions(api, { folderId }),
  )

  if (isLoading || !items) return

  return items.data.map((item) =>
    item.type === 'collection' ? (
      <CollectionItem key={item.data.id} collection={item.data} />
    ) : (
      <CollectionFolderItem key={item.data.id} folder={item.data} />
    ),
  )
}
