import type { Bookmark } from '@colette/core'
import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { BookmarkCard } from './bookmark-card'

type Props = {
  bookmark: Bookmark
}

export function SortableBookmarkCard({ bookmark }: Props) {
  const { attributes, listeners, setNodeRef, transform, transition } =
    useSortable({ id: bookmark.id, data: bookmark })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners}>
      <BookmarkCard bookmark={bookmark} />
    </div>
  )
}
