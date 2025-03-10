import { EditSubscriptionModal } from './components/edit-subscription-modal'
import { EntryList } from './components/entry-list'
import { UnsubscribeAlert } from './components/unsubscribe-alert'
import {
  getSubscriptionOptions,
  listSubscriptionEntriesOptions,
} from '@colette/query'
import { useAPI } from '@colette/util'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { ExternalLink, ListChecks, Pencil, Trash2 } from 'lucide-react'
import { type FC, useEffect } from 'react'
import { useParams } from 'wouter'
import { AlertDialog, Dialog } from '~/components/dialog'
import { AlertDialogTrigger } from '~/components/ui/alert-dialog'
import { Button } from '~/components/ui/button'
import { DialogTrigger } from '~/components/ui/dialog'

export const SubscriptionPage: FC = () => {
  const api = useAPI()
  const { id } = useParams<{ id: string }>()

  const subscriptionQuery = useQuery(getSubscriptionOptions(api, id))
  const entriesQuery = useInfiniteQuery(
    listSubscriptionEntriesOptions(api, { feedId: id }),
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
          {subscriptionQuery.data.title}
        </h1>
        <div className="flex gap-2">
          <Button asChild variant="secondary">
            <a
              href={subscriptionQuery.data.feed.link}
              target="_blank"
              rel="noreferrer"
            >
              <ExternalLink />
              Open Link
            </a>
          </Button>
          <Dialog>
            {(close) => (
              <>
                <DialogTrigger asChild>
                  <Button variant="secondary">
                    <Pencil />
                    Edit
                  </Button>
                </DialogTrigger>
                <EditSubscriptionModal
                  subscription={subscriptionQuery.data}
                  close={close}
                />
              </>
            )}
          </Dialog>
          <Button variant="secondary">
            <ListChecks />
            Mark as Read
          </Button>
          <AlertDialog>
            {(close) => (
              <>
                <AlertDialogTrigger asChild>
                  <Button variant="destructive">
                    <Trash2 />
                    Unsubscribe
                  </Button>
                </AlertDialogTrigger>
                <UnsubscribeAlert
                  subscription={subscriptionQuery.data}
                  close={close}
                />
              </>
            )}
          </AlertDialog>
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
