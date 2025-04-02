import { SubscriptionItem } from './subscription-item'
import { listSubscriptionsOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { FC } from 'react'

export const SubscriptionList: FC = () => {
  const api = useAPI()

  const query = useQuery(listSubscriptionsOptions(api, { withFeed: true }))

  if (query.isLoading || !query.data) return

  return (
    <div className="flex flex-col gap-4 px-8">
      {query.data.data.map((details) => (
        <SubscriptionItem key={details.subscription.id} details={details} />
      ))}
    </div>
  )
}
