import type { Feed } from '@colette/core'
import { deleteFeedOptions } from '@colette/query'
import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@colette/react-ui/components/ui/alert-dialog'
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
    deleteFeedOptions(feed.id, context.api, {
      onSuccess: async () => {
        close()

        if (typeof params === 'object' && params.id === feed.id) {
          await navigate({
            to: '/feeds',
          })
        }

        await context.queryClient.invalidateQueries({
          queryKey: ['feeds'],
        })
      },
    }),
  )

  return (
    <AlertDialogContent className="max-w-md p-6">
      <AlertDialogHeader>
        <AlertDialogTitle className="line-clamp-1">
          Unsubscribe from {feed.title ?? feed.originalTitle}?
        </AlertDialogTitle>
        <AlertDialogDescription>
          Are you sure you want to unsubscribe? This action cannot be undone.
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter className="mt-8">
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction onClick={() => unsubscribe()} disabled={isPending}>
          Submit
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  )
}
