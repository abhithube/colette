import { EditSubscriptionModal } from './components/edit-subscription-modal'
import { EntryList } from './components/entry-list'
import { UnsubscribeAlert } from './components/unsubscribe-alert'
import {
  getSubscriptionOptions,
  listSubscriptionEntriesOptions,
} from '@colette/query'
import { Button, Dialog } from '@colette/ui'
import { useAPI } from '@colette/util'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { ExternalLink, ListChecks, Pencil, Trash2 } from 'lucide-react'
import { useEffect } from 'react'
import { useParams } from 'wouter'

export const SubscriptionPage = () => {
  const api = useAPI()
  const { id } = useParams<{ id: string }>()

  const subscriptionQuery = useQuery(
    getSubscriptionOptions(api, id, { withFeed: true }),
  )
  const entriesQuery = useInfiniteQuery(
    listSubscriptionEntriesOptions(api, { subscriptionId: id }),
  )

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [id])

  if (
    subscriptionQuery.isLoading ||
    !subscriptionQuery.data ||
    entriesQuery.isLoading ||
    !entriesQuery.data
  )
    return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="line-clamp-1 text-3xl font-medium">
          {subscriptionQuery.data.subscription.title}
        </h1>
        <div className="flex gap-2">
          <Button asChild variant="secondary">
            <a
              href={subscriptionQuery.data.feed!.link}
              target="_blank"
              rel="noreferrer"
            >
              <ExternalLink />
              Open Link
            </a>
          </Button>
          <Dialog.Root>
            <Dialog.Trigger asChild>
              <Button variant="secondary">
                <Pencil />
                Edit
              </Button>
            </Dialog.Trigger>
            <Dialog.Context>
              {(dialogProps) => (
                <EditSubscriptionModal
                  details={subscriptionQuery.data}
                  close={() => dialogProps.setOpen(false)}
                />
              )}
            </Dialog.Context>
          </Dialog.Root>
          <Button variant="secondary">
            <ListChecks />
            Mark as Read
          </Button>
          <Dialog.Root>
            <Dialog.Trigger asChild>
              <Button variant="destructive">
                <Trash2 />
                Unsubscribe
              </Button>
            </Dialog.Trigger>
            <Dialog.Context>
              {(dialogProps) => (
                <UnsubscribeAlert
                  subscription={subscriptionQuery.data.subscription}
                  close={() => dialogProps.setOpen(false)}
                />
              )}
            </Dialog.Context>
          </Dialog.Root>
        </div>
      </div>
      <main>
        <EntryList
          entries={entriesQuery.data.pages.flatMap((page) => page.data)}
          hasMore={entriesQuery.hasNextPage}
          fetchMore={entriesQuery.fetchNextPage}
        />
      </main>
    </>
  )
}
