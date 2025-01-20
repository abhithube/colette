import { getFeedOptions, listFeedEntriesOptions } from '@colette/query'
import {
  AlertDialog,
  AlertDialogTrigger,
} from '@colette/react-ui/components/ui/alert-dialog'
import { Button } from '@colette/react-ui/components/ui/button'
import { Dialog, DialogTrigger } from '@colette/react-ui/components/ui/dialog'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { CircleX, ExternalLink, ListChecks, Pencil } from 'lucide-react'
import { useEffect, useState } from 'react'
import { EditFeedModal } from './-components/edit-feed-modal'
import { FeedEntryGrid } from './-components/feed-entry-grid'
import { UnsubscribeAlert } from './-components/unsubscribe-alert'

export const Route = createFileRoute('/_private/feeds/$id')({
  loader: async ({ context, params }) => {
    const feedOptions = getFeedOptions(params.id, context.api)

    const feedEntryOptions = listFeedEntriesOptions(
      {
        feedId: params.id,
        hasRead: false,
      },
      context.api,
    )

    await context.queryClient.ensureQueryData(feedOptions)

    return {
      feedOptions,
      feedEntryOptions,
    }
  },
  component: Component,
})

function Component() {
  const { id } = Route.useParams()
  const { feedOptions, feedEntryOptions } = Route.useLoaderData()

  const [isEditModalOpen, setEditModalOpen] = useState(false)
  const [isUnsubscribeAlertOpen, setUnsubscribeAlertOpen] = useState(false)

  const { data: feed } = useQuery(feedOptions)
  const {
    data: feedEntries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(feedEntryOptions)

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [id])

  if (!feed || !feedEntries) return

  return (
    <>
      <div className="sticky top-0 z-10 flex justify-between bg-background p-8">
        <h1 className="line-clamp-1 font-medium text-3xl">
          {feed.title ?? feed.originalTitle}
        </h1>
        <div className="flex gap-2">
          <Button asChild variant="secondary">
            <a href={feed.link} target="_blank" rel="noreferrer">
              <ExternalLink />
              Open Link
            </a>
          </Button>
          <Dialog open={isEditModalOpen} onOpenChange={setEditModalOpen}>
            <DialogTrigger asChild>
              <Button variant="secondary">
                <Pencil />
                Edit
              </Button>
            </DialogTrigger>
            <EditFeedModal feed={feed} close={() => setEditModalOpen(false)} />
          </Dialog>
          <Button variant="secondary">
            <ListChecks />
            Mark as Read
          </Button>
          <AlertDialog
            open={isUnsubscribeAlertOpen}
            onOpenChange={setUnsubscribeAlertOpen}
          >
            <AlertDialogTrigger asChild>
              <Button variant="destructive">
                <CircleX />
                Unsubscribe
              </Button>
            </AlertDialogTrigger>
            <UnsubscribeAlert
              feed={feed}
              close={() => setUnsubscribeAlertOpen(false)}
            />
          </AlertDialog>
        </div>
      </div>
      <main>
        <FeedEntryGrid
          feedEntries={feedEntries.pages.flatMap((page) => page.data)}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
    </>
  )
}
