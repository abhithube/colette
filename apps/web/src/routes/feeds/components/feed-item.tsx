import { EditFeedModal } from './edit-feed-modal'
import { Feed } from '@colette/core'
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
import { UnsubscribeAlert } from '~/sidebar/library/feeds/unsubscribe-alert'

export const FeedItem: FC<{ feed: Feed }> = (props) => {
  return (
    <Card key={props.feed.id} className="flex items-center justify-between">
      <CardHeader>
        <CardTitle>{props.feed.title}</CardTitle>
        <CardDescription className="flex items-center gap-2">
          <Favicon url={props.feed.link} />
          {new URL(props.feed.link).hostname}
        </CardDescription>
      </CardHeader>
      <CardFooter className="p-3">
        <Button asChild variant="ghost">
          <a href={props.feed.link} target="_blank" rel="noreferrer">
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
              <EditFeedModal feed={props.feed} close={close} />
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
              <UnsubscribeAlert feed={props.feed} close={close} />
            </>
          )}
        </AlertDialog>
      </CardFooter>
    </Card>
  )
}
