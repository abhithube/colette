import type { FeedEntry } from '@colette/core'
import { Separator } from '@colette/react-ui/components/ui/separator'
import groupBy from 'object.groupby'
import { useInView } from 'react-intersection-observer'
import { FeedEntryCard } from './feed-entry-card'

type Props = {
  feedEntries: FeedEntry[]
  hasMore: boolean
  loadMore?: () => void
}

export function FeedEntryGrid({ feedEntries, hasMore, loadMore }: Props) {
  const day = 1000 * 60 * 60 * 24
  const date = Date.now()
  const today = date - day
  const lastWeek = date - day * 7
  const lastMonth = date - day * 30
  const lastYear = date - day * 365

  const list = Object.entries(
    groupBy(feedEntries, (item: FeedEntry) => {
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

  const { ref } = useInView({
    threshold: 0,
    onChange: (inView) => inView && loadMore && loadMore(),
  })

  return (
    <div className="space-y-6 px-8 pb-8">
      {list.map(([title, feedEntries]) => (
        <div key={title} className="space-y-6">
          <div className="flex items-center space-x-4">
            <Separator className="flex-1" />
            <span className="font-medium text-sm">{title}</span>
            <Separator className="flex-1" />
          </div>
          <div className="container space-y-4">
            {feedEntries.map((feedEntry, i) => (
              <div
                key={feedEntry.id}
                ref={hasMore && i === feedEntries.length - 1 ? ref : undefined}
              >
                <FeedEntryCard feedEntry={feedEntry} />
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  )
}
