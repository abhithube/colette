import { EditBookmarkModal } from './edit-bookmark-modal'
import type { Bookmark } from '@colette/core'
import { formatRelativeDate } from '@colette/util'
import { ExternalLink, Pencil } from 'lucide-react'
import { type FC } from 'react'
import { Dialog } from '~/components/dialog'
import { Favicon } from '~/components/favicon'
import { Thumbnail } from '~/components/thumbnail'
import { Button } from '~/components/ui/button'
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '~/components/ui/card'
import { DialogTrigger } from '~/components/ui/dialog'
import { Separator } from '~/components/ui/separator'

export const BookmarkCard: FC<{ bookmark: Bookmark }> = (props) => {
  return (
    <Card className="overflow-hidden">
      <Thumbnail
        src={
          props.bookmark.archivedUrl ?? props.bookmark.thumbnailUrl ?? undefined
        }
        alt={props.bookmark.title}
      />
      <div className="flex flex-col pb-2">
        <CardHeader className="py-4">
          <CardTitle className="line-clamp-1" title={props.bookmark.title}>
            {props.bookmark.title}
          </CardTitle>
        </CardHeader>
        <CardContent className="flex justify-between">
          <div className="flex h-4 space-x-2">
            <div className="flex h-4 gap-2 text-sm font-semibold">
              <div className="flex gap-2">
                <Favicon url={props.bookmark.link} />
                {props.bookmark.author && (
                  <span className="truncate" title={props.bookmark.author}>
                    {props.bookmark.author}
                  </span>
                )}
              </div>
              {props.bookmark.publishedAt && (
                <>
                  <Separator orientation="vertical" />
                  <span title={new Date(props.bookmark.publishedAt).toString()}>
                    {formatRelativeDate(props.bookmark.publishedAt)}
                  </span>
                </>
              )}
            </div>
          </div>
        </CardContent>
      </div>
      <CardFooter className="py-0 pb-4">
        <Button asChild variant="ghost" title="Open in new tab">
          <a href={props.bookmark.link} target="_blank" rel="noreferrer">
            <ExternalLink />
          </a>
        </Button>
        <Dialog>
          {(close) => (
            <>
              <DialogTrigger asChild>
                <Button variant="ghost" title="Edit bookmark">
                  <Pencil />
                </Button>
              </DialogTrigger>
              <EditBookmarkModal bookmark={props.bookmark} close={close} />
            </>
          )}
        </Dialog>
      </CardFooter>
    </Card>
  )
}
