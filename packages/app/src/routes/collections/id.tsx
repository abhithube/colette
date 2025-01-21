import { getCollectionOptions, listBookmarksOptions } from '@colette/query'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'
import { useParams } from 'wouter'
import { useAPI } from '../../lib/api-context'
import { BookmarkGrid } from './components/bookmark-grid'

export const CollectionPage: FC = () => {
  const api = useAPI()
  const { id } = useParams<{ id: string }>()

  const { data: collection } = useQuery(getCollectionOptions(id, api))
  const {
    data: bookmarks,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(listBookmarksOptions({ collectionId: id }, api))

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!collection || !bookmarks) return

  return (
    <>
      <div className="sticky top-0 z-10 flex justify-between bg-background p-8">
        <h1 className="font-medium text-3xl">{collection.title}</h1>
      </div>
      <main>
        <BookmarkGrid
          bookmarks={bookmarks.pages.flatMap((page) => page.data) ?? []}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
