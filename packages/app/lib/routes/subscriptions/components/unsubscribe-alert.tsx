import { Subscription } from '@colette/core/types'
import { useDeleteSubscriptionMutation } from '@colette/query'
import { getRouteApi, useParams } from '@colette/router'
import { Button, Dialog } from '@colette/ui'

const routeApi = getRouteApi('/layout')

export const UnsubscribeAlert = (props: {
  subscription: Subscription
  close: () => void
}) => {
  const navigate = routeApi.useNavigate()

  const params = useParams({
    strict: false,
  })

  const deleteSubscription = useDeleteSubscriptionMutation(
    props.subscription.id,
  )

  function onDelete() {
    deleteSubscription.mutate(undefined, {
      onSuccess: () => {
        props.close()

        if (params.subscriptionId === props.subscription.id) {
          navigate({
            to: '/subscriptions',
          })
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
