import { EditSubscriptionModal } from './edit-subscription-modal'
import { SubscriptionDetails } from '@colette/core'
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

export const SubscriptionItem: FC<{ details: SubscriptionDetails }> = (
  props,
) => {
  return (
    <Card
      key={props.details.subscription.id}
      className="flex items-center justify-between"
    >
      <CardHeader>
        <CardTitle>{props.details.subscription.title}</CardTitle>
        <CardDescription className="flex items-center gap-2">
          <Favicon url={props.details.feed!.link} />
          {new URL(props.details.feed!.link).hostname}
        </CardDescription>
      </CardHeader>
      <CardFooter className="p-3">
        <Button asChild variant="ghost">
          <a href={props.details.feed!.link} target="_blank" rel="noreferrer">
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
              <EditSubscriptionModal details={props.details} close={close} />
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
                subscription={props.details.subscription}
                close={close}
              />
            </>
          )}
        </AlertDialog>
      </CardFooter>
    </Card>
  )
}
