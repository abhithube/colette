import { useAPI } from '../../lib/api-context'
import { BookmarkGrid } from './components/bookmark-grid'
import { getCollectionOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'
import { useParams } from 'wouter'

export const CollectionPage: FC = () => {
  const api = useAPI()
  const { id } = useParams<{ id: string }>()

  const { data: collection } = useQuery(getCollectionOptions(id, api))

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!collection) return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">{collection.title}</h1>
      </div>
      <main>
        <BookmarkGrid query={{ collectionId: id }} />
      </main>
    </>
  )
}
