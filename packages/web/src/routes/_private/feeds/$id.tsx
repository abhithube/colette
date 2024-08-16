import {
  Header,
  HeaderActionGroup,
  HeaderActionItem,
  HeaderTitle,
} from '@/components/header'
import { Icon } from '@/components/icon'
import { Dialog } from '@/components/ui/dialog'
import {
  ensureInfiniteQueryData,
  getFeedOptions,
  listFeedEntriesOptions,
} from '@colette/query'
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
      context.profile.id,
      context.api,
    )

    await Promise.all([
      context.queryClient.ensureQueryData(feedOptions),
      ensureInfiniteQueryData(context.queryClient, feedEntryOptions as any),
    ])

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

  const { data: feed } = useQuery(feedOptions)
  const {
    data: feedEntries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(feedEntryOptions)

  const [isEditModalOpen, setEditModalOpen] = useState(false)
  const [isUnsubscribeAlertOpen, setUnsubscribeAlertOpen] = useState(false)

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [id])

  if (!feed || !feedEntries) return

  return (
    <>
      <Header>
        <HeaderTitle>{feed.title ?? feed.originalTitle}</HeaderTitle>
        <HeaderActionGroup>
          <HeaderActionItem asChild>
            <a href={feed.link} target="_blank" rel="noreferrer">
              <Icon value={ExternalLink} />
              <span>Open Link</span>
            </a>
          </HeaderActionItem>
          <HeaderActionItem onClick={() => setEditModalOpen(true)}>
            <Icon value={Pencil} />
            <span>Edit</span>
          </HeaderActionItem>
          <HeaderActionItem>
            <Icon value={ListChecks} />
            <span>Mark as Read</span>
          </HeaderActionItem>
          <HeaderActionItem
            variant="destructive"
            onClick={() => setUnsubscribeAlertOpen(true)}
          >
            <Icon value={CircleX} />
            <span>Unsubscribe</span>
          </HeaderActionItem>
        </HeaderActionGroup>
      </Header>
      <main>
        <FeedEntryGrid
          feedEntries={feedEntries.pages.flatMap((page) => page.data)}
          hasMore={hasNextPage}
          loadMore={fetchNextPage}
        />
      </main>
      <UnsubscribeAlert
        feed={feed}
        isOpen={isUnsubscribeAlertOpen}
        setOpen={setUnsubscribeAlertOpen}
      />
      <Dialog open={isEditModalOpen} onOpenChange={setEditModalOpen}>
        <EditFeedModal feed={feed} close={() => setEditModalOpen(false)} />
      </Dialog>
    </>
  )
}
