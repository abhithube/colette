import { CreateSubscriptionModal } from './components/create-form/create-subscription-modal'
import { SubscriptionList } from './components/subscription-list'
import { Button, Dialog } from '@colette/ui'
import { Plus } from 'lucide-react'
import { useEffect } from 'react'

export const SubscriptionsPage = () => {
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="text-3xl font-medium">Subscriptions</h1>
        <Dialog.Root>
          <Dialog.Trigger asChild>
            <Button variant="secondary">
              <Plus />
              New
            </Button>
          </Dialog.Trigger>
          <Dialog.Context>
            {(dialogProps) => (
              <CreateSubscriptionModal
                close={() => dialogProps.setOpen(false)}
              />
            )}
          </Dialog.Context>
        </Dialog.Root>
      </div>
      <main>
        <SubscriptionList />
      </main>
    </>
  )
}
