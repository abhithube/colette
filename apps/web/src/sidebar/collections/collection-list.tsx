import { CollectionItem } from './collection-item'
import { listCollectionsOptions } from '@colette/query'
import { Sidebar } from '@colette/ui'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'

export const CollectionList = () => {
  const api = useAPI()

  const query = useQuery(listCollectionsOptions(api))

  if (query.isLoading || !query.data) return

  return (
    <Sidebar.Menu>
      {query.data.data.map((collection) => (
        <CollectionItem key={collection.id} collection={collection} />
      ))}
    </Sidebar.Menu>
  )
}
