import { listCollectionsOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import type { FC } from 'react'
import { useAPI } from '../../lib/api-context'
import { CollectionItem } from './collection-item'

export const CollectionList: FC = () => {
  const api = useAPI()

  const { data: collections, isLoading } = useQuery(listCollectionsOptions(api))

  if (isLoading || !collections) return

  return collections.data.map((collection) => (
    <CollectionItem key={collection.id} collection={collection} />
  ))
}
