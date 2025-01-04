import { listBookmarksOptions } from '@colette/solid-query'
import { createInfiniteQuery } from '@tanstack/solid-query'
import { type Component, For, onCleanup, onMount } from 'solid-js'
import { useAPI } from '~/lib/api-context'
import { BookmarkCard } from './bookmark-card'

export const BookmarkGrid: Component<{
  collectionId?: string
}> = (props) => {
  const query = createInfiniteQuery(() =>
    listBookmarksOptions({ collectionId: props.collectionId }, useAPI()),
  )

  const allBookmarks = () => query.data?.pages.flatMap((page) => page.data)

  let target: HTMLDivElement | undefined

  onMount(() => {
    const observer = new IntersectionObserver(
      async (entries) => {
        if (entries.at(0)?.isIntersecting && query.hasNextPage) {
          query.fetchNextPage()
        }
      },
      {
        rootMargin: '200px',
      },
    )

    if (target) {
      observer.observe(target)
    }

    onCleanup(() => {
      if (target) {
        observer.unobserve(target)
      }
    })
  })

  return (
    <div class="space-y-8 px-8 pb-8">
      <div class="grid grid-cols-3 gap-4">
        <For
          each={allBookmarks()}
          fallback={
            <div class="text-muted-foreground">
              No bookmarks in this collection.
            </div>
          }
        >
          {(item) => <BookmarkCard bookmark={item} />}
        </For>
      </div>
      <div ref={target} class="sr-only" />
    </div>
  )
}
