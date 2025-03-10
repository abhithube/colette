import type { Subscription } from '@colette/core'
import { useDeleteSubscriptionMutation } from '@colette/query'
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
  subscription: Subscription
  close: () => void
}> = (props) => {
  const params = useParams<{ id?: string }>()

  const deleteSubscription = useDeleteSubscriptionMutation(
    props.subscription.id,
  )

  function onDelete() {
    deleteSubscription.mutate(undefined, {
      onSuccess: () => {
        props.close()

        if (params.id === props.subscription.id) {
          navigate('/feeds')
        }
      },
    })
  }

  return (
    <AlertDialogContent>
      <AlertDialogTitle className="line-clamp-1">
        Unsubscribe from{' '}
        <span className="text-primary">{props.subscription.title}</span>
      </AlertDialogTitle>
      <AlertDialogDescription>
        Are you sure you want to unsubscribe? This action cannot be undone.
      </AlertDialogDescription>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction
          disabled={deleteSubscription.isPending}
          onClick={onDelete}
        >
          Confirm
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  )
}
