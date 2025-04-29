import { SubscriptionItem } from './subscription-item'
import { listSubscriptionsOptions } from '@colette/query'
import { useQuery } from '@tanstack/react-query'
import { getRouteApi } from '@tanstack/react-router'

const routeApi = getRouteApi('/layout/subscriptions')

export const SubscriptionList = () => {
  const context = routeApi.useRouteContext()

  const query = useQuery(
    listSubscriptionsOptions(context.api, { withFeed: true }),
  )

  if (query.isLoading || !query.data) return

  return (
    <div className="flex flex-col gap-4 px-8">
      {query.data.data.map((details) => (
        <SubscriptionItem key={details.subscription.id} details={details} />
      ))}
    </div>
  )
}
