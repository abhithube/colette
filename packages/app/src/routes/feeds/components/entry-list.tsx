import { useIntersectionObserver } from '../../../lib/use-intersection-observer'
import { EntryCard } from './entry-card'
import type { FeedEntry } from '@colette/core'
import { Separator } from '@colette/react-ui/components/ui/separator'
import { type FC } from 'react'

export const EntryList: FC<{
  entries: FeedEntry[]
  hasMore: boolean
  loadMore?: () => void
}> = (props) => {
  const day = 1000 * 60 * 60 * 24
  const date = Date.now()
  const today = date - day
  const lastWeek = date - day * 7
  const lastMonth = date - day * 30
  const lastYear = date - day * 365

  const list = Object.entries(
    Object.groupBy(props.entries, (item: FeedEntry) => {
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

  const target = useIntersectionObserver({
    options: {
      rootMargin: '200px',
    },
    onChange: (isIntersecting) =>
      isIntersecting && props.hasMore && props.loadMore?.(),
  })

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
