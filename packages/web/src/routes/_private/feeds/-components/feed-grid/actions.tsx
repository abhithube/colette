import { HStack, IconButton } from '@colette/components'
import type { Feed } from '@colette/core'
import { Pencil, Pin, Trash2 } from 'lucide-react'

type Props = {
  feed: Feed
}

export function FeedRowActions({ feed }: Props) {
  return (
    <HStack>
      <IconButton variant="ghost">
        <Pin fill={feed.pinned ? 'black' : 'none'} />
      </IconButton>
      <IconButton variant="ghost">
        <Pencil />
      </IconButton>
      <IconButton variant="ghost" colorPalette="red">
        <Trash2 />
      </IconButton>
    </HStack>
  )
}
