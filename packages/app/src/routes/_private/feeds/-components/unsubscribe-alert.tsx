import type { Feed } from '@colette/core'
import { deleteFeedOptions } from '@colette/query'
import { Button, Dialog, Flex } from '@colette/ui'
import { useMutation } from '@tanstack/react-query'
import { useMatchRoute, useNavigate } from '@tanstack/react-router'
import { Route } from '../../feeds'

type Props = {
  feed: Feed
  close: () => void
}

export function UnsubscribeAlert({ feed, close }: Props) {
  const context = Route.useRouteContext()

  const navigate = useNavigate()

  const matchRoute = useMatchRoute()
  const params = matchRoute({ to: '/feeds/$id' })

  const { mutateAsync: unsubscribe, isPending } = useMutation(
    deleteFeedOptions(
      feed.id,
      {
        onSuccess: async () => {
          close()

          if (typeof params === 'object' && params.id === feed.id) {
            await navigate({
              to: '/feeds',
            })
          }

          await context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'feeds'],
          })
        },
      },
      context.api,
    ),
  )

  return (
    <Dialog.Content maxW="md" p={6}>
      <Dialog.Title lineClamp={1}>
        Unsubscribe from {feed.title ?? feed.originalTitle}?
      </Dialog.Title>
      <Dialog.Description>
        Are you sure you want to unsubscribe? This action cannot be undone.
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
