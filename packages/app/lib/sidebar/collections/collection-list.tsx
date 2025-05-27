import { CollectionItem } from './collection-item'
import { listCollectionsOptions } from '@colette/query'
import { Sidebar } from '@colette/ui'
import { useQuery } from '@tanstack/react-query'

export const CollectionList = () => {
  const query = useQuery(listCollectionsOptions())

  if (query.isLoading || !query.data) return

  return (
    <Sidebar.Menu>
      {query.data.data.map((collection) => (
        <CollectionItem key={collection.id} collection={collection as never} />
      ))}
    </Sidebar.Menu>
  )
}
