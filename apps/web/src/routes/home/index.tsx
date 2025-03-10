import { EntryList } from '../subscriptions/components/entry-list'
import { listSubscriptionEntriesOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useInfiniteQuery } from '@tanstack/react-query'
import { type FC, useEffect } from 'react'

export const HomePage: FC = () => {
  const api = useAPI()

  const query = useInfiniteQuery(
    listSubscriptionEntriesOptions(api, { hasRead: false }),
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
          entries={query.data.pages.flatMap((page) => page.data)}
          hasMore={query.hasNextPage}
          fetchMore={query.fetchNextPage}
        />
      </main>
    </>
  )
}
