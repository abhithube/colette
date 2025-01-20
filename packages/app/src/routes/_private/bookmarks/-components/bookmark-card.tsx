import type { Bookmark } from '@colette/core'
import { Favicon } from '@colette/react-ui/components/favicon'
import { Button } from '@colette/react-ui/components/ui/button'
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@colette/react-ui/components/ui/card'
import { Dialog, DialogTrigger } from '@colette/react-ui/components/ui/dialog'
import { Separator } from '@colette/react-ui/components/ui/separator'
import { ExternalLink, Pencil } from 'lucide-react'
import { useState } from 'react'
import { formatRelativeDate } from '../../../../lib/utils'
import { EditBookmarkModal } from './edit-bookmark-modal'

type Props = {
  bookmark: Bookmark
}

export function BookmarkCard({ bookmark }: Props) {
  const [isOpen, setOpen] = useState(false)

  return (
    <Card>
      <img
        className="aspect-video bg-background object-cover"
        src={
          bookmark.thumbnailUrl ?? 'https://placehold.co/320x180/black/black'
        }
        alt={bookmark.title}
        loading="lazy"
      />
      <div className="flex flex-col pb-2">
        <CardHeader className="py-4">
          <CardTitle className="line-clamp-1" title={bookmark.title}>
            {bookmark.title}
          </CardTitle>
        </CardHeader>
        <CardContent className="flex justify-between">
          <div className="flex h-4 space-x-2">
            <div className="flex h-4 gap-2 font-semibold text-sm">
              <div className="flex gap-2">
                <Favicon url={bookmark.link} />
                {bookmark.author && (
                  <span className="truncate" title={bookmark.author}>
                    {bookmark.author}
                  </span>
                )}
              </div>
              {bookmark.publishedAt && (
                <>
                  <Separator orientation="vertical" />
                  <span title={new Date(bookmark.publishedAt).toString()}>
                    {formatRelativeDate(bookmark.publishedAt)}
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
            href={bookmark.link}
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
          <EditBookmarkModal bookmark={bookmark} close={() => setOpen(false)} />
        </Dialog>
      </CardFooter>
    </Card>
  )
}
