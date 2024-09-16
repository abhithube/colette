import { Dialog } from '@/components/ui/dialog'
import { Button, HStack, Heading, Icon, Link } from '@colette/components'
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
      <HStack
        pos="sticky"
        zIndex="sticky"
        top={0}
        justify="space-between"
        bg="bg.default"
        p={8}
      >
        <Heading as="h1" fontSize="3xl" fontWeight="medium">
          {feed.title ?? feed.originalTitle}
        </Heading>
        <HStack>
          <Button asChild variant="subtle">
            <Link href={feed.link} target="_blank" textDecoration="none">
              <Icon>
                <ExternalLink />
              </Icon>
              Open Link
            </Link>
          </Button>
          <Button variant="subtle" onClick={() => setEditModalOpen(true)}>
            <Icon>
              <Pencil />
            </Icon>
            Edit
          </Button>
          <Button variant="subtle">
            <Icon>
              <ListChecks />
            </Icon>
            Mark as Read
          </Button>
          <Button
            variant="subtle"
            colorPalette="red"
            onClick={() => setUnsubscribeAlertOpen(true)}
          >
            <Icon>
              <CircleX />
            </Icon>
            Unsubscribe
          </Button>
        </HStack>
      </HStack>
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
