import type { Subscription } from '@colette/core'
import { useDeleteSubscriptionMutation } from '@colette/query'
import { Button, Dialog } from '@colette/ui'
import { useParams } from 'wouter'
import { navigate } from 'wouter/use-browser-location'

export const UnsubscribeAlert = (props: {
  subscription: Subscription
  close: () => void
}) => {
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
    <Dialog.Content>
      <Dialog.Title className="line-clamp-1">
        Unsubscribe from{' '}
        <span className="text-primary">{props.subscription.title}</span>
      </Dialog.Title>
      <Dialog.Description>
        Are you sure you want to unsubscribe? This action cannot be undone.
      </Dialog.Description>
      <Dialog.Footer>
        <Button variant="secondary" onClick={props.close}>
          Cancel
        </Button>
        <Button disabled={deleteSubscription.isPending} onClick={onDelete}>
          Confirm
        </Button>
      </Dialog.Footer>
    </Dialog.Content>
  )
}
