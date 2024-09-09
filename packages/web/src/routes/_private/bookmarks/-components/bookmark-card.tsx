import { Favicon } from '@/components/favicon'
import { formatRelativeDate } from '@/lib/utils'
import {
  Button,
  Card,
  Dialog,
  Divider,
  HStack,
  Icon,
  IconButton,
  Link,
  Text,
  css,
} from '@colette/components'
import type { Bookmark } from '@colette/core'
import { ExternalLink, Pencil } from 'lucide-react'
import { useState } from 'react'
import { EditBookmarkModal } from './edit-bookmark-modal'

type Props = {
  bookmark: Bookmark
}

export function BookmarkCard({ bookmark }: Props) {
  const [isEditModalOpen, setEditModalOpen] = useState(false)

  return (
    <Card.Root>
      <img
        className={css({
          aspectRatio: 16 / 9,
          bg: 'bg.default',
          objectFit: 'cover',
        })}
        src={
          bookmark.thumbnailUrl ?? 'https://placehold.co/320x180/black/black'
        }
        alt={bookmark.title}
        loading="lazy"
      />
      <div className="flex flex-col pb-2">
        <Card.Header py={4}>
          <Card.Title lineClamp={1} title={bookmark.title}>
            {bookmark.title}
          </Card.Title>
        </Card.Header>
        <Card.Body className="flex justify-between">
          <div className="flex h-4 space-x-2">
            <HStack gap={2} h={4} fontSize="sm" fontWeight="semibold">
              <HStack gap={2}>
                <Favicon domain={new URL(bookmark.link).hostname} />
                {bookmark.author && (
                  <Text as="span" truncate title={bookmark.author}>
                    {bookmark.author}
                  </Text>
                )}
              </HStack>
              {bookmark.publishedAt && (
                <>
                  <Divider orientation="vertical" />
                  <Text
                    as="span"
                    title={new Date(bookmark.publishedAt).toString()}
                  >
                    {formatRelativeDate(bookmark.publishedAt)}
                  </Text>
                </>
              )}
            </HStack>
          </div>
        </Card.Body>
      </div>
      <Card.Footer py={0} pb={4}>
        <Button asChild variant="ghost" title="Open in new tab">
          <Link href={bookmark.link} target="_blank">
            <Icon color="fg.muted">
              <ExternalLink />
            </Icon>
          </Link>
        </Button>
        <Dialog.Root
          open={isEditModalOpen}
          onOpenChange={(e) => setEditModalOpen(e.open)}
        >
          <Dialog.Trigger asChild>
            <IconButton variant="ghost" color="fg.muted" title="Edit bookmark">
              <Pencil />
            </IconButton>
          </Dialog.Trigger>
          <Dialog.Backdrop />
          <Dialog.Positioner>
            <EditBookmarkModal
              bookmark={bookmark}
              close={() => setEditModalOpen(false)}
            />
          </Dialog.Positioner>
        </Dialog.Root>
      </Card.Footer>
    </Card.Root>
  )
}
