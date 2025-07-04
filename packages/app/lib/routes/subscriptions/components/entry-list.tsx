import { EntryCard } from './entry-card'
import { SubscriptionEntryDetails } from '@colette/core/types'
import { Separator } from '@colette/ui'
import { useIntersectionObserver } from '@colette/util'

export const EntryList = (props: {
  entries: SubscriptionEntryDetails[]
  hasMore: boolean
  fetchMore: () => void
}) => {
  const target = useIntersectionObserver({
    options: {
      rootMargin: '200px',
    },
    onChange: (isIntersecting) =>
      isIntersecting && props.hasMore && props.fetchMore(),
  })

  const day = 1000 * 60 * 60 * 24
  const date = Date.now()
  const today = date - day
  const lastWeek = date - day * 7
  const lastMonth = date - day * 30
  const lastYear = date - day * 365

  const list = Object.entries(
    Object.groupBy(props.entries, (item: SubscriptionEntryDetails) => {
      const publishedAt = Date.parse(item.feedEntry!.publishedAt)
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
          <div className="grid grid-cols-3 gap-4">
            {entries.map((details) => (
              <EntryCard
                key={details.subscriptionEntry.feedEntryId}
                details={details}
              />
            ))}
          </div>
        </div>
      ))}
      <div ref={target} />
    </div>
  )
}
