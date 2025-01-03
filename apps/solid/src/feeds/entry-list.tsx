import { listFeedEntriesOptions } from '@colette/solid-query'
import { createInfiniteQuery } from '@tanstack/solid-query'
import { type Component, For } from 'solid-js'
import { Separator } from '~/components/ui/separator'
import { useAPI } from '~/lib/api-context'
import { EntryCard } from './entry-card'

const day = 1000 * 60 * 60 * 24
const date = Date.now()
const today = date - day
const lastWeek = date - day * 7
const lastMonth = date - day * 30
const lastYear = date - day * 365

export const EntryList: Component<{
  feedId?: string
}> = (props) => {
  const query = createInfiniteQuery(() =>
    listFeedEntriesOptions({ feedId: props.feedId }, useAPI()),
  )

  const allEntries = () => query.data?.pages.flatMap((page) => page.data)

  const list = () => {
    if (!allEntries()) return undefined

    return Object.entries(
      Object.groupBy(allEntries()!, (item) => {
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
                {(item) => (
                  <div>
                    <EntryCard entry={item} />
                  </div>
                )}
              </For>
            </div>
          </div>
        )}
      </For>
    </div>
  )
}
