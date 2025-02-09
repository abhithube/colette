import { listLibraryItemsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { useState, type FC } from 'react'

export const LibraryPage: FC = () => {
  const api = useAPI()

  const [folderId] = useState<string | undefined>(undefined)

  const { data: libraryItems, isLoading } = useQuery(
    listLibraryItemsOptions({ folderId }, api),
  )

  if (isLoading || !libraryItems) return

  return (
    <div>
      <h1>Library</h1>
      <code>{JSON.stringify(libraryItems)}</code>
    </div>
  )
}
