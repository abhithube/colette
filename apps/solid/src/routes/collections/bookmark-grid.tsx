import { BookmarkCard } from './bookmark-card'
import { listBookmarksOptions } from '@colette/query'
import { createInfiniteQuery } from '@tanstack/solid-query'
import { type Component, For, onCleanup, onMount } from 'solid-js'
import { useAPI } from '~/lib/api-context'

export const BookmarkGrid: Component<{
  collectionId?: string
}> = (props) => {
  const api = useAPI()

  const query = createInfiniteQuery(() =>
    listBookmarksOptions({ collectionId: props.collectionId }, api),
  )

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
          each={query.data?.pages.flatMap((page) => page.data)}
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
