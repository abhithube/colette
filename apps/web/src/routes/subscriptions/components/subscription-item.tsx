import { EditSubscriptionModal } from './edit-subscription-modal'
import { EditSubscriptionTagsModal } from './edit-subscription-tags-modal'
import { SubscriptionDetails } from '@colette/core'
import { Button, Card, Dialog, Favicon } from '@colette/ui'
import { ExternalLink, Pencil, Tag, Trash2 } from 'lucide-react'
import { UnsubscribeAlert } from '~/routes/subscriptions/components/unsubscribe-alert'

export const SubscriptionItem = (props: { details: SubscriptionDetails }) => {
  return (
    <Card.Root
      key={props.details.subscription.id}
      className="flex items-center justify-between"
    >
      <Card.Header className="gap-2">
        <Card.Title>{props.details.subscription.title}</Card.Title>
        <Card.Description className="flex items-center gap-2">
          <Favicon src={props.details.feed!.link} />
          {new URL(props.details.feed!.link).hostname}
        </Card.Description>
      </Card.Header>
      <Card.Footer className="p-3">
        <Button asChild variant="ghost">
          <a href={props.details.feed!.link} target="_blank" rel="noreferrer">
            <ExternalLink />
          </a>
        </Button>

        <Dialog.Root lazyMount>
          <Dialog.Trigger asChild>
            <Button variant="ghost">
              <Pencil />
            </Button>
          </Dialog.Trigger>
          <Dialog.Context>
            {(dialogProps) => (
              <EditSubscriptionModal
                subscription={props.details.subscription}
                close={() => dialogProps.setOpen(false)}
              />
            )}
          </Dialog.Context>
        </Dialog.Root>

        <Dialog.Root lazyMount>
          <Dialog.Trigger asChild>
            <Button variant="ghost">
              <Tag />
            </Button>
          </Dialog.Trigger>
          <Dialog.Context>
            {(dialogProps) => (
              <EditSubscriptionTagsModal
                details={props.details}
                close={() => dialogProps.setOpen(false)}
              />
            )}
          </Dialog.Context>
        </Dialog.Root>

        <Dialog.Root lazyMount>
          <Dialog.Trigger asChild>
            <Button variant="ghost">
              <Trash2 />
            </Button>
          </Dialog.Trigger>
          <Dialog.Context>
            {(dialogProps) => (
              <UnsubscribeAlert
                subscription={props.details.subscription}
                close={() => dialogProps.setOpen(false)}
              />
            )}
          </Dialog.Context>
        </Dialog.Root>
      </Card.Footer>
    </Card.Root>
  )
}
