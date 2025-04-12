import { SubscriptionList } from './components/subscription-list'
import { useEffect } from 'react'

export const SubscriptionsPage = () => {
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">Subscriptions</h1>
      </div>
      <main>
        <SubscriptionList />
      </main>
    </>
  )
}
