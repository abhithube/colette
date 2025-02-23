import { EditFeedModal } from './components/edit-feed-modal'
import { EntryList } from './components/entry-list'
import { UnsubscribeAlert } from './components/unsubscribe-alert'
import { getFeedOptions } from '@colette/query'
import { useAPI } from '@colette/util'
import { useQuery } from '@tanstack/react-query'
import { ExternalLink, ListChecks, Pencil, Trash2 } from 'lucide-react'
import { type FC, useEffect } from 'react'
import { useParams } from 'wouter'
import { AlertDialog, Dialog } from '~/components/dialog'
import { AlertDialogTrigger } from '~/components/ui/alert-dialog'
import { Button } from '~/components/ui/button'
import { DialogTrigger } from '~/components/ui/dialog'

export const FeedPage: FC = () => {
  const api = useAPI()
  const { id } = useParams<{ id: string }>()

  const { data: feed } = useQuery(getFeedOptions(api, id))

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [id])

  if (!feed) return

  return (
    <>
      <div className="bg-background sticky top-0 z-10 flex justify-between p-8">
        <h1 className="line-clamp-1 text-3xl font-medium">{feed.title}</h1>
        <div className="flex gap-2">
          <Button asChild variant="secondary">
            <a href={feed.link} target="_blank" rel="noreferrer">
              <ExternalLink />
              Open Link
            </a>
          </Button>
          <Dialog>
            {(close) => (
              <>
                <DialogTrigger asChild>
                  <Button variant="secondary">
                    <Pencil />
                    Edit
                  </Button>
                </DialogTrigger>
                <EditFeedModal feed={feed} close={close} />
              </>
            )}
          </Dialog>
          <Button variant="secondary">
            <ListChecks />
            Mark as Read
          </Button>
          <AlertDialog>
            {(close) => (
              <>
                <AlertDialogTrigger asChild>
                  <Button variant="destructive">
                    <Trash2 />
                    Unsubscribe
                  </Button>
                </AlertDialogTrigger>
                <UnsubscribeAlert feed={feed} close={close} />
              </>
            )}
          </AlertDialog>
        </div>
      </div>
      <main>
        <EntryList query={{ feedId: id }} />
      </main>
    </>
  )
}
