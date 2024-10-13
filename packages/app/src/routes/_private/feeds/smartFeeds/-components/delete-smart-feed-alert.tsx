import type { SmartFeed } from '@colette/core'
import { deleteSmartFeedOptions } from '@colette/query'
import { Button, Dialog, Flex } from '@colette/ui'
import { useMutation } from '@tanstack/react-query'
import { useMatchRoute, useNavigate } from '@tanstack/react-router'
import { Route } from '../../../feeds'

type Props = {
  smartFeed: SmartFeed
  close: () => void
}

export function DeleteSmartFeedAlert({ smartFeed, close }: Props) {
  const context = Route.useRouteContext()

  const navigate = useNavigate()

  const matchRoute = useMatchRoute()
  const params = matchRoute({ to: '/feeds/smartFeeds/$id' })

  const { mutateAsync: unsubscribe, isPending } = useMutation(
    deleteSmartFeedOptions(
      smartFeed.id,
      {
        onSuccess: async () => {
          close()

          if (typeof params === 'object' && params.id === smartFeed.id) {
            await navigate({
              to: '/feeds',
            })
          }

          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'smartFeeds'],
          })
        },
      },
      context.api,
    ),
  )

  return (
    <Dialog.Content maxW="md" p={6}>
      <Dialog.Title lineClamp={1}>Delete {smartFeed.title}?</Dialog.Title>
      <Dialog.Description>
        Are you sure you want to delete this smart feed? This action cannot be
        undone.
      </Dialog.Description>
      <Flex justify="end" spaceX={4} mt={8}>
        <Dialog.CloseTrigger asChild>
          <Button variant="outline">Cancel</Button>
        </Dialog.CloseTrigger>
        <Button loading={isPending} onClick={() => unsubscribe()}>
          Submit
        </Button>
      </Flex>
    </Dialog.Content>
  )
}
