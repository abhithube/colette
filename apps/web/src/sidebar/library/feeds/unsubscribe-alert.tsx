import type { Feed } from '@colette/core'
import { useDeleteFeedMutation } from '@colette/query'
import type { FC } from 'react'
import { useParams } from 'wouter'
import { navigate } from 'wouter/use-browser-location'
import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogTitle,
} from '~/components/ui/alert-dialog'

export const UnsubscribeAlert: FC<{
  feed: Feed
  close: () => void
}> = (props) => {
  const params = useParams<{ id?: string }>()

  const deleteFeed = useDeleteFeedMutation(props.feed.id)

  return (
    <AlertDialogContent>
      <AlertDialogTitle className="line-clamp-1">
        Unsubscribe from{' '}
        <span className="text-primary">{props.feed.title}</span>
      </AlertDialogTitle>
      <AlertDialogDescription>
        Are you sure you want to unsubscribe? This action cannot be undone.
      </AlertDialogDescription>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction
          disabled={deleteFeed.isPending}
          onClick={() =>
            deleteFeed.mutate(undefined, {
              onSuccess: () => {
                props.close()

                if (params.id === props.feed.id) {
                  navigate('/feeds')
                }
              },
            })
          }
        >
          Confirm
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  )
}
