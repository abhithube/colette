import { EditBookmarkModal } from './edit-bookmark-modal'
import type { BookmarkDetails } from '@colette/core'
import { Button, Card, Dialog, Favicon, Menu } from '@colette/ui'
import { Separator } from '@colette/ui'
import { formatRelativeDate } from '@colette/util'
import { ExternalLink, MoreHorizontal, Pencil } from 'lucide-react'
import { useState } from 'react'
import { Thumbnail } from '~/components/thumbnail'

export const BookmarkCard = (props: { details: BookmarkDetails }) => {
  const [isEditDialogOpen, setEditDialogOpen] = useState(false)

  return (
    <Card.Root className="overflow-hidden pt-0">
      <Thumbnail
        src={
          props.details.bookmark.archivedUrl ??
          props.details.bookmark.thumbnailUrl ??
          undefined
        }
        alt={props.details.bookmark.title}
      />
      <Card.Header>
        <Card.Title
          className="line-clamp-1 leading-tight"
          title={props.details.bookmark.title}
        >
          {props.details.bookmark.title}
        </Card.Title>
      </Card.Header>
      <Card.Footer className="justify-between">
        <div className="flex h-4 items-center gap-2 text-sm font-medium">
          <Favicon src={props.details.bookmark.link} />
          {props.details.bookmark.author && (
            <span className="truncate" title={props.details.bookmark.author}>
              {props.details.bookmark.author}
            </span>
          )}
          {props.details.bookmark.publishedAt && (
            <>
              <Separator orientation="vertical" />
              <span
                title={new Date(props.details.bookmark.publishedAt).toString()}
              >
                {formatRelativeDate(props.details.bookmark.publishedAt)}
              </span>
            </>
          )}
        </div>
        <Menu.Root
          lazyMount
          onSelect={(details) => {
            if (details.value === 'edit-metadata') {
              setEditDialogOpen(true)
            }
          }}
        >
          <Menu.Trigger asChild>
            <Button className="size-7" variant="ghost" size="icon">
              <MoreHorizontal className="size-4" />
              <span className="sr-only">Entry actions</span>
            </Button>
          </Menu.Trigger>
          <Menu.Content>
            <Menu.Item value="open-link" asChild>
              <a
                href={props.details.bookmark.link}
                target="_blank"
                rel="noreferrer"
              >
                <ExternalLink />
                Open link
              </a>
            </Menu.Item>
            <Menu.Item value="edit-metadata">
              <Pencil />
              Edit metadata
            </Menu.Item>
          </Menu.Content>
        </Menu.Root>
        <Dialog.Root
          lazyMount
          open={isEditDialogOpen}
          onOpenChange={(details) => setEditDialogOpen(details.open)}
        >
          <Dialog.Context>
            {(dialogProps) => (
              <EditBookmarkModal
                details={props.details}
                close={() => dialogProps.setOpen(false)}
              />
            )}
          </Dialog.Context>
        </Dialog.Root>
      </Card.Footer>
    </Card.Root>
  )
}
