import type { Bookmark } from '@colette/openapi'
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import {
  SortableContext,
  arrayMove,
  sortableKeyboardCoordinates,
} from '@dnd-kit/sortable'
import { useState } from 'react'
import { useInView } from 'react-intersection-observer'
import { BookmarkCard } from './bookmark-card'
import { SortableBookmarkCard } from './sortable-bookmark-card'

type Props = {
  bookmarks: Bookmark[]
  hasMore: boolean
  loadMore?: () => void
  created?: Bookmark
}

export function BookmarkGrid({
  bookmarks: initialBookmarks,
  hasMore = false,
  loadMore,
  created,
}: Props) {
  const [bookmarks, setBookmarks] = useState(
    created
      ? initialBookmarks.filter((v) => v.id !== created.id)
      : initialBookmarks,
  )
  const [active, setActive] = useState<Bookmark | null>(null)

  const { ref } = useInView({
    threshold: 0,
    onChange: (inView) => inView && loadMore && loadMore(),
  })

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  )

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={({ active }) => {
        if (active.data.current) {
          setActive(active.data.current as Bookmark)
        }
      }}
      onDragEnd={({ active, over }) => {
        if (!over || active.id === over.id) return

        const from = bookmarks.findIndex(
          (bookmark) => bookmark.id === active.id,
        )
        const to = bookmarks.findIndex((bookmark) => bookmark.id === over.id)

        setBookmarks(arrayMove(bookmarks, from, to))
      }}
    >
      <SortableContext items={bookmarks}>
        <div className="grid grid-cols-1 gap-4 px-8 pb-8 md:grid-cols-2 lg:grid-cols-3">
          {created && (
            <div className="rounded-lg border-2 border-secondary">
              <BookmarkCard bookmark={created} />
            </div>
          )}
          {bookmarks.map((bookmark, i) => (
            <div
              key={bookmark.id}
              ref={hasMore && i === bookmarks.length - 1 ? ref : undefined}
            >
              <SortableBookmarkCard bookmark={bookmark} />
            </div>
          ))}
        </div>
      </SortableContext>
      <DragOverlay>
        {active ? <BookmarkCard bookmark={active} /> : null}
      </DragOverlay>
    </DndContext>
  )
}
