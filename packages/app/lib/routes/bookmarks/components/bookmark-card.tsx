import { Thumbnail } from '../../../components/thumbnail'
import { EditBookmarkModal } from './edit-bookmark-modal'
import { EditBookmarkTagsModal } from './edit-bookmark-tags-modal'
import { BookmarkDetails } from '@colette/core/types'
import { Button, Card, Dialog, Favicon, Menu } from '@colette/ui'
import { Separator } from '@colette/ui'
import { formatRelativeDate } from '@colette/util'
import { ExternalLink, MoreHorizontal, Pencil, Tag } from 'lucide-react'
import { useState } from 'react'

export const BookmarkCard = (props: { details: BookmarkDetails }) => {
  const [isMetadataDialogOpen, setMetadataDialogOpen] = useState(false)
  const [isTagsDialogOpen, setTagsDialogOpen] = useState(false)

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
            switch (details.value) {
              case 'edit-metadata':
                setMetadataDialogOpen(true)
                break
              case 'edit-tags':
                setTagsDialogOpen(true)
                break
            }
          }}
        >
          <Menu.Trigger asChild>
            <Button variant="ghost" size="icon">
              <MoreHorizontal />
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
              Edit Metadata
            </Menu.Item>
            <Menu.Item value="edit-tags">
              <Tag />
              Edit Tags
            </Menu.Item>
          </Menu.Content>
        </Menu.Root>
        <Dialog.Root
          lazyMount
          open={isMetadataDialogOpen}
          onOpenChange={(details) => setMetadataDialogOpen(details.open)}
        >
          <Dialog.Context>
            {(dialogProps) => (
              <EditBookmarkModal
                bookmark={props.details.bookmark}
                close={() => dialogProps.setOpen(false)}
              />
            )}
          </Dialog.Context>
        </Dialog.Root>
        <Dialog.Root
          lazyMount
          open={isTagsDialogOpen}
          onOpenChange={(details) => setTagsDialogOpen(details.open)}
        >
          <Dialog.Context>
            {(dialogProps) => (
              <EditBookmarkTagsModal
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
