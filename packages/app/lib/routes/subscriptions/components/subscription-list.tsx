import { SubscriptionItem } from './subscription-item'
import { listSubscriptionsOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'

export const SubscriptionList = () => {
  const query = useQuery(listSubscriptionsOptions({ withFeed: true }))

  if (query.isLoading || !query.data) return

  return (
    <div className="flex flex-col gap-4 px-8">
      {query.data.items.map((details) => (
        <SubscriptionItem key={details.subscription.id} details={details} />
      ))}
    </div>
  )
}
