import type { Bookmark } from '@colette/core'
import { updateBookmarkOptions } from '@colette/query'
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
import { useMutation } from '@tanstack/react-query'
import { useState } from 'react'
import { useInView } from 'react-intersection-observer'
import { Route } from '../../bookmarks'
import { BookmarkCard } from './bookmark-card'
import { SortableBookmarkCard } from './sortable-bookmark-card'

type Props = {
  bookmarks: Bookmark[]
  setBookmarks: React.Dispatch<React.SetStateAction<Bookmark[]>>
  hasMore: boolean
  loadMore?: () => void
  created?: Bookmark
}

export function BookmarkGrid({
  bookmarks,
  setBookmarks,
  hasMore = false,
  loadMore,
  created,
}: Props) {
  const context = Route.useRouteContext()

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

  const filtered = created
    ? bookmarks.filter((v) => v.id !== created.id)
    : bookmarks

  const { mutateAsync: updateBookmark } = useMutation(
    updateBookmarkOptions(
      {
        onSettled: () =>
          context.queryClient.invalidateQueries({
            queryKey: ['profiles', context.profile.id, 'bookmarks'],
          }),
      },
      context.api,
    ),
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
      onDragEnd={async ({ active, over }) => {
        if (!over || active.id === over.id) return

        const from = filtered.findIndex((bookmark) => bookmark.id === active.id)
        const to = filtered.findIndex((bookmark) => bookmark.id === over.id)

        setBookmarks(arrayMove(filtered, from, to))

        await updateBookmark({
          id: active.id as string,
          body: {
            sortIndex: over.data.current?.sortIndex,
          },
        })
      }}
    >
      <SortableContext items={filtered}>
        <div className="grid grid-cols-1 gap-4 px-8 pb-8 md:grid-cols-2 lg:grid-cols-3">
          {created && (
            <div className="rounded-lg border-2 border-secondary">
              <BookmarkCard bookmark={created} />
            </div>
          )}
          {filtered.map((bookmark, i) => (
            <div
              key={bookmark.id}
              ref={hasMore && i === filtered.length - 1 ? ref : undefined}
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
