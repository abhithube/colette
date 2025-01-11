import type { Feed } from '@colette/core'
import { updateFeedOptions } from '@colette/query'
import { Dialog, HStack, IconButton } from '@colette/ui'
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
    updateFeedOptions(context.api, {
      onSuccess: async (data) => {
        await context.queryClient.setQueryData(['feeds', feed.id], data)
        await context.queryClient.invalidateQueries({
          queryKey: ['feeds'],
        })
      },
    }),
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
