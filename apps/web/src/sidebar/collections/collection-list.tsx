import { CollectionItem } from './collection-item'
import { listCollectionsOptions } from '@colette/query'
import { Sidebar } from '@colette/ui'
import { useQuery } from '@tanstack/react-query'
import { getRouteApi } from '@tanstack/react-router'

const routeApi = getRouteApi('/layout')

export const CollectionList = () => {
  const context = routeApi.useRouteContext()

  const query = useQuery(listCollectionsOptions(context.api))

  if (query.isLoading || !query.data) return

  return (
    <Sidebar.Menu>
      {query.data.data.map((collection) => (
        <CollectionItem key={collection.id} collection={collection} />
      ))}
    </Sidebar.Menu>
  )
}
