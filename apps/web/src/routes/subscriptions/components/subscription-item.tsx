import { EditSubscriptionModal } from './edit-subscription-modal'
import { Subscription } from '@colette/core'
import { ExternalLink, Pencil, Trash2 } from 'lucide-react'
import { FC } from 'react'
import { AlertDialog, Dialog } from '~/components/dialog'
import { Favicon } from '~/components/favicon'
import { AlertDialogTrigger } from '~/components/ui/alert-dialog'
import { Button } from '~/components/ui/button'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardFooter,
} from '~/components/ui/card'
import { DialogTrigger } from '~/components/ui/dialog'
import { UnsubscribeAlert } from '~/routes/subscriptions/components/unsubscribe-alert'

export const SubscriptionItem: FC<{ subscription: Subscription }> = (props) => {
  return (
    <Card
      key={props.subscription.id}
      className="flex items-center justify-between"
    >
      <CardHeader>
        <CardTitle>{props.subscription.title}</CardTitle>
        <CardDescription className="flex items-center gap-2">
          <Favicon url={props.subscription.feed.link} />
          {new URL(props.subscription.feed.link).hostname}
        </CardDescription>
      </CardHeader>
      <CardFooter className="p-3">
        <Button asChild variant="ghost">
          <a
            href={props.subscription.feed.link}
            target="_blank"
            rel="noreferrer"
          >
            <ExternalLink />
          </a>
        </Button>
        <Dialog>
          {(close) => (
            <>
              <DialogTrigger asChild>
                <Button variant="ghost">
                  <Pencil />
                </Button>
              </DialogTrigger>
              <EditSubscriptionModal
                subscription={props.subscription}
                close={close}
              />
            </>
          )}
        </Dialog>
        <AlertDialog>
          {(close) => (
            <>
              <AlertDialogTrigger asChild>
                <Button variant="ghost">
                  <Trash2 />
                </Button>
              </AlertDialogTrigger>
              <UnsubscribeAlert
                subscription={props.subscription}
                close={close}
              />
            </>
          )}
        </AlertDialog>
      </CardFooter>
    </Card>
  )
}
