import { formatRelativeDate } from '../../../lib/utils'
import { EditBookmarkModal } from './edit-bookmark-modal'
import type { Bookmark } from '@colette/core'
import { ExternalLink, Pencil } from 'lucide-react'
import { type FC, useState } from 'react'
import { Favicon } from '~/components/favicon'
import { Button } from '~/components/ui/button'
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '~/components/ui/card'
import { Dialog, DialogTrigger } from '~/components/ui/dialog'
import { Separator } from '~/components/ui/separator'

export const BookmarkCard: FC<{ bookmark: Bookmark }> = (props) => {
  const [isOpen, setOpen] = useState(false)

  return (
    <Card className="overflow-hidden">
      <img
        className="bg-background aspect-video object-cover"
        src={
          props.bookmark.thumbnailUrl ??
          'https://placehold.co/320x180/black/black'
        }
        alt={props.bookmark.title}
        loading="lazy"
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
          <a
            className="text-muted"
            href={props.bookmark.link}
            target="_blank"
            rel="noreferrer"
          >
            <ExternalLink />
          </a>
        </Button>
        <Dialog open={isOpen} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button
              className="text-muted"
              variant="ghost"
              title="Edit bookmark"
            >
              <Pencil />
            </Button>
          </DialogTrigger>
          <EditBookmarkModal
            bookmark={props.bookmark}
            close={() => setOpen(false)}
          />
        </Dialog>
      </CardFooter>
    </Card>
  )
}
