import { useAPI } from '../../../lib/api-context'
import { useIntersectionObserver } from '../../../lib/use-intersection-observer'
import { EntryCard } from './entry-card'
import type { FeedEntry, FeedEntryListQuery } from '@colette/core'
import { listFeedEntriesOptions } from '@colette/query'
import { Separator } from '@colette/react-ui/components/ui/separator'
import { useInfiniteQuery } from '@tanstack/react-query'
import { type FC } from 'react'

export const EntryList: FC<{ query: FeedEntryListQuery }> = (props) => {
  const api = useAPI()

  const { data, isLoading, hasNextPage, fetchNextPage } = useInfiniteQuery(
    listFeedEntriesOptions(props.query, api),
  )

  const target = useIntersectionObserver({
    options: {
      rootMargin: '200px',
    },
    onChange: (isIntersecting) =>
      isIntersecting && hasNextPage && fetchNextPage(),
  })

  if (isLoading || !data) return

  const feedEntries = data.pages.flatMap((page) => page.data)

  const day = 1000 * 60 * 60 * 24
  const date = Date.now()
  const today = date - day
  const lastWeek = date - day * 7
  const lastMonth = date - day * 30
  const lastYear = date - day * 365

  const list = Object.entries(
    Object.groupBy(feedEntries, (item: FeedEntry) => {
      const publishedAt = Date.parse(item.publishedAt)
      return publishedAt > today
        ? 'Today'
        : publishedAt > lastWeek
          ? 'This Week'
          : publishedAt > lastMonth
            ? 'This Month'
            : publishedAt > lastYear
              ? 'This Year'
              : 'This Lifetime'
    }),
  )

  return (
    <div className="space-y-6 px-8 pb-8">
      {list.map(([title, entries]) => (
        <div key={title} className="space-y-6">
          <div className="flex items-center space-x-4">
            <Separator className="flex-1" />
            <span className="text-sm font-medium">{title}</span>
            <Separator className="flex-1" />
          </div>
          <div className="container space-y-4">
            {entries.map((entry) => (
              <EntryCard key={entry.id} entry={entry} />
            ))}
          </div>
        </div>
      ))}
      <div ref={target} />
    </div>
  )
}
