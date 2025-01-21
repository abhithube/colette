import { listFeedEntriesOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'
import { useAPI } from '../../lib/api-context'
import { EntryGrid } from './components/entry-grid'

export const HomePage: FC = () => {
  const api = useAPI()

  const {
    data: feedEntries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(listFeedEntriesOptions({ hasRead: false }, api))

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!feedEntries) return

  return (
    <>
      <div className="sticky top-0 z-10 flex justify-between bg-background p-8">
        <h1 className="font-medium text-3xl">All Feeds</h1>
      </div>
      <main>
        <EntryGrid
          entries={feedEntries.pages.flatMap((page) => page.data)}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
