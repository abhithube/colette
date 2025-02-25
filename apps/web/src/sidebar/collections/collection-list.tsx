import { CollectionItem } from './collection-item'
import { listCollectionsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'
import { SidebarMenu } from '~/components/ui/sidebar'

export const CollectionList: FC = () => {
  const api = useAPI()

  const query = useQuery(listCollectionsOptions(api))

  if (query.isLoading || !query.data) return

  return (
    <SidebarMenu>
      {query.data.data.map((collection) => (
        <CollectionItem key={collection.id} collection={collection} />
      ))}
    </SidebarMenu>
  )
}
