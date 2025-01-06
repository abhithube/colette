import { listFeedEntriesOptions } from '@colette/query'
import { createInfiniteQuery } from '@tanstack/solid-query'
import { type Component, For, onCleanup, onMount } from 'solid-js'
import { Separator } from '~/components/ui/separator'
import { useAPI } from '~/lib/api-context'
import { EntryCard } from './entry-card'

export const EntryList: Component<{
  feedId?: string
}> = (props) => {
  const api = useAPI()

  const query = createInfiniteQuery(() =>
    listFeedEntriesOptions({ feedId: props.feedId }, api),
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

  const list = () => {
    const day = 1000 * 60 * 60 * 24
    const date = Date.now()
    const today = date - day
    const lastWeek = date - day * 7
    const lastMonth = date - day * 30
    const lastYear = date - day * 365

    const entries = query.data?.pages.flatMap((page) => page.data)
    if (entries) {
      return Object.entries(
        Object.groupBy(entries, (item) => {
          const publishedAt = Date.parse(item.publishedAt)
          return publishedAt > today
            ? 'Today'
            : publishedAt > lastWeek
              ? 'This Week'
              : publishedAt > lastMonth
                ? 'This Month'
                : publishedAt > lastYear
                  ? 'This Year'
                  : 'Older'
        }),
      )
    }
  }

  return (
    <div class="space-y-8 px-8 pb-8">
      <For each={list()}>
        {([title, entries]) => (
          <div class="space-y-8">
            <div class="flex items-center gap-8">
              <Separator class="flex-1" />
              <span class="font-medium text-sm">{title}</span>
              <Separator class="flex-1" />
            </div>
            <div class="container space-y-4">
              <For each={entries}>
                {(item) => {
                  return <EntryCard entry={item} />
                }}
              </For>
            </div>
          </div>
        )}
      </For>
      <div ref={target} class="sr-only" />
    </div>
  )
}
