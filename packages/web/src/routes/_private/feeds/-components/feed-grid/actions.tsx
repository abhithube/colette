import { Dialog, HStack, IconButton } from '@colette/components'
import type { Feed } from '@colette/core'
import { updateFeedOptions } from '@colette/query'
import { useMutation } from '@tanstack/react-query'
import { Pencil, Pin, Trash2 } from 'lucide-react'
import { Route } from '../../manage'
import { EditFeedModal } from '../edit-feed-modal'
import { UnsubscribeAlert } from '../unsubscribe-alert'

type Props = {
  feed: Feed
}

export function FeedRowActions({ feed }: Props) {
  const context = Route.useRouteContext()

  const { mutateAsync: updateFeed } = useMutation(
    updateFeedOptions(
      {
        onSuccess: async (data) => {
          close()

          await context.queryClient.setQueryData(['feeds', feed.id], data)
          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'feeds'],
          })
        },
      },
      context.api,
    ),
  )

  return (
    <HStack>
      <IconButton
        variant="ghost"
        onClick={() =>
          updateFeed({
            id: feed.id,
            body: {
              pinned: !feed.pinned,
            },
          })
        }
      >
        <Pin fill={feed.pinned ? 'black' : 'none'} />
      </IconButton>
      <Dialog.Root>
        <Dialog.Trigger asChild>
          <IconButton variant="ghost">
            <Pencil />
          </IconButton>
        </Dialog.Trigger>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Context>
            {({ setOpen }) => (
              <EditFeedModal feed={feed} close={() => setOpen(false)} />
            )}
          </Dialog.Context>
        </Dialog.Positioner>
      </Dialog.Root>
      <Dialog.Root>
        <Dialog.Trigger asChild>
          <IconButton variant="ghost" colorPalette="red">
            <Trash2 />
          </IconButton>
        </Dialog.Trigger>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Context>
            {({ setOpen }) => (
              <UnsubscribeAlert feed={feed} close={() => setOpen(false)} />
            )}
          </Dialog.Context>
        </Dialog.Positioner>
      </Dialog.Root>
    </HStack>
  )
}
