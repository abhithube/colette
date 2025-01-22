import { useAPI } from '../../lib/api-context'
import { EntryList } from './components/entry-list'
import { listFeedEntriesOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'

export const ArchivedPage: FC = () => {
  const api = useAPI()

  const {
    data: feedEntries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(listFeedEntriesOptions({ hasRead: true }, api))

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (!feedEntries) return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">Archived</h1>
      </div>
      <main>
        <EntryList
          entries={feedEntries.pages.flatMap((page) => page.data)}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
