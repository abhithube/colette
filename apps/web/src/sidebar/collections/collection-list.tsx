import { CollectionItem } from './collection-item'
import { listCollectionsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'
import { SidebarMenu } from '~/components/ui/sidebar'

export const CollectionList: FC = () => {
  const api = useAPI()

  const { data: collections, isLoading } = useQuery(listCollectionsOptions(api))

  if (isLoading || !collections) return

  return (
    <SidebarMenu>
      {collections.data.map((collection) => (
        <CollectionItem key={collection.id} collection={collection} />
      ))}
    </SidebarMenu>
  )
}
