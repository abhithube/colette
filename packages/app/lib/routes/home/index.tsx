import { EntryList } from '../subscriptions/components/entry-list'
import { listSubscriptionEntriesOptions } from '@colette/query'
import { useInfiniteQuery } from '@tanstack/react-query'
import { useEffect } from 'react'

export const HomePage = () => {
  const query = useInfiniteQuery(
    listSubscriptionEntriesOptions({ hasRead: false }),
  )

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  if (query.isLoading || !query.data) return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">All Feeds</h1>
      </div>
      <main>
        <EntryList
          entries={query.data.pages.flatMap((page) => page.items)}
          hasMore={query.hasNextPage}
          fetchMore={query.fetchNextPage}
        />
      </main>
    </>
  )
}
