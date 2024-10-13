import {
  ensureInfiniteQueryData,
  getSmartFeedOptions,
  listFeedEntriesOptions,
} from '@colette/query'
import { Button, Dialog, HStack, Heading, Icon } from '@colette/ui'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { CircleX, Pencil } from 'lucide-react'
import { useEffect } from 'react'
import { FeedEntryGrid } from '../-components/feed-entry-grid'
import { DeleteSmartFeedAlert } from './-components/delete-smart-feed-alert'
import { EditSmartFeedModal } from './-components/edit-feed-modal'

export const Route = createFileRoute('/_private/feeds/smartFeeds/$id')({
  loader: async ({ context, params }) => {
    const smartFeedOptions = getSmartFeedOptions(params.id, context.api)

    const feedEntryOptions = listFeedEntriesOptions(
      {
        smartFeedId: params.id,
        hasRead: false,
      },
      context.profile.id,
      context.api,
    )

    await Promise.all([
      context.queryClient.ensureQueryData(smartFeedOptions),
      ensureInfiniteQueryData(context.queryClient, feedEntryOptions as any),
    ])

    return {
      smartFeedOptions,
      feedEntryOptions,
    }
  },
  component: Component,
})

function Component() {
  const { id } = Route.useParams()
  const { smartFeedOptions, feedEntryOptions } = Route.useLoaderData()

  const { data: smartFeed } = useQuery(smartFeedOptions)
  const {
    data: feedEntries,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery(feedEntryOptions)

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [id])

  if (!smartFeed || !feedEntries) return

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
        <Heading as="h1" fontSize="3xl" fontWeight="medium" lineClamp={1}>
          {smartFeed.title}
        </Heading>
        <HStack>
          <Dialog.Root>
            <Dialog.Trigger asChild>
              <Button variant="subtle">
                <Icon>
                  <Pencil />
                </Icon>
                Edit
              </Button>
            </Dialog.Trigger>
            <Dialog.Backdrop />
            <Dialog.Positioner>
              <Dialog.Context>
                {({ setOpen }) => (
                  <EditSmartFeedModal
                    smartFeed={smartFeed}
                    close={() => setOpen(false)}
                  />
                )}
              </Dialog.Context>
            </Dialog.Positioner>
          </Dialog.Root>
          <Dialog.Root>
            <Dialog.Trigger asChild>
              <Button variant="subtle" colorPalette="red">
                <Icon>
                  <CircleX />
                </Icon>
                Delete
              </Button>
            </Dialog.Trigger>
            <Dialog.Backdrop />
            <Dialog.Positioner>
              <Dialog.Context>
                {({ setOpen }) => (
                  <DeleteSmartFeedAlert
                    smartFeed={smartFeed}
                    close={() => setOpen(false)}
                  />
                )}
              </Dialog.Context>
            </Dialog.Positioner>
          </Dialog.Root>
        </HStack>
      </HStack>
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
