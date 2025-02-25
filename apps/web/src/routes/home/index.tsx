import { EntryList } from '../feeds/components/entry-list'
import { listFeedEntriesOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useInfiniteQuery } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'

export const HomePage: FC = () => {
  const api = useAPI()

  const entriesQuery = useInfiniteQuery(
    listFeedEntriesOptions(api, { hasRead: false }),
  )

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (entriesQuery.isLoading || !entriesQuery.data) return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">All Feeds</h1>
      </div>
      <main>
        <EntryList
          entries={entriesQuery.data.pages.flatMap((page) => page.data)}
          hasMore={entriesQuery.hasNextPage}
          fetchMore={entriesQuery.fetchNextPage}
        />
      </main>
    </>
  )
}
